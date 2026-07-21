use std::ops::Deref;

use cargo_tools::cargo_make::{MakefileTask, MakefileTasks, ParseError, parse_tasks};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_viewless::Task;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::recent_items::RecentItems;
use crate::{
    environment::makefile_task_context,
    extension::{
        CommandBinding, send_file_changed,
        tasks::cargo_make::{
            command::{Command, register_cargo_make_commands},
            tree_provider::CargoMakeTreeProviderHandler,
        },
    },
    quick_pick::SelectInput,
    runtime::{
        CHANNEL_CAPACITY, TsFileWatcher, VsCodeTask, exec_vs_code, execute_task,
        file_exists_vs_code, get_state_vs_code, persist_state_vs_code, set_makefile_context,
    },
};
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(MakefileTasks),
    CargoMakeNotInstalled(String),
    NoMakefile,
    FailedToRetrieve(String),
    CargoCommandEmpty(String),
}

impl MakefileTasksUpdate {
    fn from_parse_result(res: Result<MakefileTasks, ParseError>) -> Self {
        match res {
            Ok(metadata) => Self::New(metadata),
            Err(e) => match e {
                ParseError::CargoMakeNotInstalled(e) => Self::CargoMakeNotInstalled(e),
                ParseError::NoMakefile => Self::NoMakefile,
                ParseError::Exec(e) | ParseError::FailedToRetrieve(e) => Self::FailedToRetrieve(e),
                ParseError::CargoCommandEmpty(e) => Self::CargoCommandEmpty(e.to_string()),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    MakefileChanged,
    MakefileTasksChanged(MakefileTasksUpdate),
    SettingsChanged(SettingsUpdate),
    Cmd(Command),
}

pub enum Event {
    AddPinned(MakefileTask),
    TreeChanged(CargoMakeTreeProviderHandler),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    TaskFilter(String),
    CategoryFilter(Vec<String>),
    AddPinned(MakefileTask),
    RecordRun(String),
}

#[derive(Debug)]
pub struct CargoMake {
    makefile_tasks: MakefileTasks,
    settings: Settings,
    _cmds: Vec<CommandBinding>,
    _file_watcher: TsFileWatcher,
    root_dir: String,
    cmd_tx: Sender<Command>,
}

impl CargoMake {
    /// Inits all data and update channels
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        // Init makefile updates
        let (makefile_changed_tx, makefile_changed_rx) = channel(CHANNEL_CAPACITY);
        let _file_watcher = TsFileWatcher::new(send_file_changed(makefile_changed_tx));
        _file_watcher.watch_files(vec![makefile(&root_dir)]);

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_cargo_make_commands(cmd_tx.clone());

        let settings: Settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let this = Self {
            makefile_tasks: MakefileTasks::default(),
            settings,
            _cmds,
            _file_watcher,
            root_dir,
            cmd_tx,
        };

        // makefile update and cmd will run for the lifetime of the extension
        let manifest_update = Task::stream(makefile_changed_rx).map(|()| Message::MakefileChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        // makefile_tasks is to initially parse the available makefile tasks
        let makefile_tasks = Task::future(parse_tasks(
            makefile(&this.root_dir),
            makefile_task_context(),
            exec_vs_code,
        ))
        .map(MakefileTasksUpdate::from_parse_result)
        .map(Message::MakefileTasksChanged);
        let tasks = Task::batch([manifest_update, cmd, makefile_tasks]);

        (this, tasks)
    }

    pub fn makefile_tasks(&self) -> &MakefileTasks {
        &self.makefile_tasks
    }

    pub fn name_filter(&self) -> &str {
        &self.settings.task_filter
    }

    pub fn update(&mut self, msg: Message) -> (Task<Message>, Option<Event>) {
        match msg {
            Message::MakefileTasksChanged(update) => match update {
                MakefileTasksUpdate::New(makefile_tasks) => {
                    self.settings
                        .recent_tasks
                        .remove_obsolete(&makefile_tasks, |task| &task.name);
                    self.makefile_tasks = makefile_tasks.clone();
                    let event = self.tree_changed_event();
                    (
                        Task::batch([
                            Task::future(set_makefile_context(true)).discard(),
                            Task::future(persist_state_vs_code(
                                settings_key(&self.root_dir),
                                self.settings.clone(),
                            ))
                            .discard(),
                        ]),
                        Some(event),
                    )
                }
                MakefileTasksUpdate::CargoMakeNotInstalled(e) => {
                    error!("{e}");
                    self.set_context_false()
                }
                MakefileTasksUpdate::NoMakefile => self.set_context_false(),
                // For invalid makefiles or cargo command config leave everything as is
                MakefileTasksUpdate::CargoCommandEmpty(e)
                | MakefileTasksUpdate::FailedToRetrieve(e) => {
                    error!("{e}");
                    (Task::none(), None)
                }
            },
            Message::MakefileChanged => {
                let makefile = makefile(&self.root_dir);
                let task = Task::future(async move {
                    if file_exists_vs_code(makefile.clone()).await {
                        parse_tasks(makefile, makefile_task_context(), exec_vs_code).await
                    } else {
                        Err(ParseError::NoMakefile)
                    }
                })
                .map(MakefileTasksUpdate::from_parse_result)
                .map(Message::MakefileTasksChanged);
                (task, None)
            }
            Message::SettingsChanged(update) => self.update_state(update),
            Message::Cmd(cmd) => (self.handle_cmd(cmd), None),
        }
    }

    fn set_context_false(&mut self) -> (Task<Message>, Option<Event>) {
        self.makefile_tasks = MakefileTasks::default();
        let event = self.tree_changed_event();
        (
            Task::future(set_makefile_context(false)).discard(),
            Some(event),
        )
    }

    fn update_state(&mut self, update: SettingsUpdate) -> (Task<Message>, Option<Event>) {
        let Settings {
            task_filter,
            category_filters,
            recent_tasks,
        } = &mut self.settings;

        let event = match update {
            SettingsUpdate::TaskFilter(tf) => {
                *task_filter = tf;
                Some(self.tree_changed_event())
            }
            SettingsUpdate::CategoryFilter(cf) => {
                *category_filters = cf;
                Some(self.tree_changed_event())
            }
            SettingsUpdate::AddPinned(makefile_task) => Some(Event::AddPinned(makefile_task)),
            SettingsUpdate::RecordRun(name) => {
                recent_tasks.record(name);
                None
            }
        };

        let task = Task::future(persist_state_vs_code(
            settings_key(&self.root_dir),
            self.settings.clone(),
        ))
        .discard();

        (task, event)
    }

    fn tree_changed_event(&self) -> Event {
        let tasks = self
            .makefile_tasks
            .filtered(&self.settings.task_filter, &self.settings.category_filters);
        Event::TreeChanged(CargoMakeTreeProviderHandler::new(tasks))
    }

    fn handle_cmd(&self, cmd: Command) -> Task<Message> {
        match cmd {
            Command::RunTask(task) => self.make_task_exec(task),
            Command::SelectAndRunTask => {
                let options = self
                    .settings
                    .recent_tasks
                    .apply(self.makefile_tasks.deref(), |task| &task.name);
                let input = SelectInput {
                    options,
                    current: Vec::new(),
                };
                done(async move { input.select().await.map(|task| Command::RunTask(task.name)) })
            }
            Command::SelectCategoryFilter => self.select_category_filter(),
            Command::EditCategoryFilter(category_filters) => {
                Task::done(SettingsUpdate::CategoryFilter(category_filters).into_cargo_make_msg())
            }
            Command::PinTask(task) => self
                .makefile_tasks
                .iter()
                .find(|t| t.name == task)
                .cloned()
                .map(|task| Task::done(SettingsUpdate::AddPinned(task).into_cargo_make_msg()))
                .unwrap_or(Task::none()),
        }
    }

    fn make_task_exec(&self, make_task: String) -> Task<Message> {
        match MakefileTask::try_into_process(make_task.clone(), makefile_task_context()) {
            Ok(process) => Task::batch([
                Task::future(execute_task(VsCodeTask::cargo_make(process))).discard(),
                Task::done(Message::SettingsChanged(SettingsUpdate::RecordRun(
                    make_task,
                ))),
            ]),
            Err(e) => {
                error!("{e}");
                Task::none()
            }
        }
    }

    fn select_category_filter(&self) -> Task<Message> {
        let categories = self
            .makefile_tasks
            .iter()
            .map(|task| &task.category)
            .unique();

        let current = self.settings.category_filters.clone();
        // Select all categories that are not filtered out
        let selected = categories
            .clone()
            .filter(|category| !current.contains(category))
            .cloned()
            .collect();

        let categories: Vec<_> = categories.cloned().collect();
        let input = SelectInput {
            options: categories.clone(),
            current: selected,
        };

        let cmd_tx = self.cmd_tx.clone();
        let categories_filter_update = categories.clone();
        let filter_update = move |selected: Vec<String>| {
            debug!("Received category filter update from quickpick'{selected:?}'");
            let mut tx = cmd_tx.clone();
            let categories = categories_filter_update.clone();
            spawn_local(async move {
                let selected = categories
                    .iter()
                    .filter(|category| !selected.contains(category))
                    .cloned()
                    .collect();
                debug!("Sending cargo make category filter '{selected:?}'");
                if let Err(e) = tx.send(Command::EditCategoryFilter(selected)).await {
                    error!("Failed to queue msg: {}", e);
                }
            });
        };

        Task::future(async move {
            let selected_categories = input
                .select_multiple(filter_update)
                .await
                .map(|selected| {
                    // Select all categories that are not filtered out
                    categories
                        .into_iter()
                        .filter(|category| !selected.contains(category))
                        .collect()
                })
                .unwrap_or(current);

            SettingsUpdate::CategoryFilter(selected_categories)
        })
        .map(Message::SettingsChanged)
    }
}

trait IntoCargoMakeMessage {
    fn into_cargo_make_msg(self) -> Message;
}

impl IntoCargoMakeMessage for Command {
    fn into_cargo_make_msg(self) -> Message {
        Message::Cmd(self)
    }
}

impl IntoCargoMakeMessage for SettingsUpdate {
    fn into_cargo_make_msg(self) -> Message {
        Message::SettingsChanged(self)
    }
}

fn done(
    fut: impl Future<Output = Option<impl IntoCargoMakeMessage + 'static>> + 'static,
) -> Task<Message> {
    Task::future(fut)
        .and_then(Task::done)
        .map(IntoCargoMakeMessage::into_cargo_make_msg)
}

fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.tasks.cargo_make.ui_settings")
}

fn makefile(root_dir: &str) -> String {
    format!("{root_dir}/Makefile.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    task_filter: String,
    category_filters: Vec<String>,
    #[serde(default)]
    recent_tasks: RecentItems,
}
