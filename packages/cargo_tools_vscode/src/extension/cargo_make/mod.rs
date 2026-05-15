pub mod command;
pub mod ui;

use std::iter;

use cargo_tools::cargo_make::{MakefileTask, MakefileTasks, ParseError, parse_tasks};
use futures::channel::mpsc::{Sender, channel};
use iced_viewless::Task;

use serde::{Deserialize, Serialize};

use crate::{
    extension::{
        base::{Base, send_file_changed},
        cargo_make::{
            command::{Command, register_cargo_make_commands},
            ui::{CargoMakePinnedTreeProviderHandler, CargoMakeTreeProviderHandler},
        },
    },
    runtime::{CHANNEL_CAPACITY, exec_vs_code, get_state_vs_code, persist_state_vs_code},
    vs_code_api::{
        CargoMakePinnedTreeProvider, CargoMakeTreeProvider, TsFileWatcher, log_error,
        set_makefile_context, showInformationMessage,
    },
};

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(MakefileTasks),
    CargoMakeNotInstalled(String),
    NoMakefile(String),
    FailedToRetrieve(String),
}

impl MakefileTasksUpdate {
    fn from_parse_result(res: Result<MakefileTasks, ParseError>) -> Self {
        match res {
            Ok(metadata) => Self::New(metadata),
            Err(e) => match e {
                ParseError::CargoMakeNotInstalled(e) => Self::CargoMakeNotInstalled(e),
                ParseError::NoMakefile(e) => Self::NoMakefile(e),
                ParseError::FailedToRetrieve(e) => Self::FailedToRetrieve(e),
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

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    AddPinned(MakefileTask),
    RemovePinned(usize),
    TaskFilter(String),
    CategoryFilter(Vec<String>),
}

#[derive(Debug)]
pub struct Ui {
    makefile_tasks: MakefileTasks,
    settings: Settings,
    ui: CargoMakeTreeProvider,
    ui_pinned: CargoMakePinnedTreeProvider,
    base: Base,
    cmd_tx: Sender<Command>,
}

impl Ui {
    /// Inits all data and update channels
    pub fn new(root_dir: String) -> (Self, Task<Message>) {
        // Init makefile updates
        let (makefile_changed_tx, makefile_changed_rx) = channel(CHANNEL_CAPACITY);
        let file_watcher = TsFileWatcher::new(send_file_changed(makefile_changed_tx));
        file_watcher.watch_files(vec![makefile(&root_dir)]);

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let cmds = register_cargo_make_commands(cmd_tx.clone());

        let settings: Settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let base = Base {
            cmds,
            file_watcher,
            root_dir,
        };

        let handler = CargoMakeTreeProviderHandler::new(MakefileTasks::default());
        let handler_pinned =
            CargoMakePinnedTreeProviderHandler::new(settings.pinned_makefile_tasks.clone());

        let this = Self {
            makefile_tasks: MakefileTasks::default(),
            settings,
            ui: CargoMakeTreeProvider::new(handler),
            ui_pinned: CargoMakePinnedTreeProvider::new(handler_pinned),
            base,
            cmd_tx,
        };

        // makefile update and cmd will run for the lifetime of the extension
        let manifest_update = Task::stream(makefile_changed_rx).map(|()| Message::MakefileChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        // makefile_tasks is to initially parse the available makefile tasks
        let makefile_tasks = Task::future(parse_tasks(makefile(&this.base.root_dir), exec_vs_code))
            .map(MakefileTasksUpdate::from_parse_result)
            .map(Message::MakefileTasksChanged);
        let tasks = Task::batch([manifest_update, cmd, makefile_tasks]);

        (this, tasks)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::MakefileTasksChanged(update) => match update {
                MakefileTasksUpdate::New(makefile_tasks) => {
                    self.makefile_tasks = makefile_tasks.clone();
                    self.update_ui();
                    Task::future(set_makefile_context(true)).discard()
                }
                MakefileTasksUpdate::CargoMakeNotInstalled(e)
                | MakefileTasksUpdate::NoMakefile(e) => {
                    log_error(&e);
                    self.makefile_tasks = MakefileTasks::default();
                    self.update_ui();
                    Task::future(set_makefile_context(false)).discard()
                }
                // For invalid makefiles leave everything as is
                MakefileTasksUpdate::FailedToRetrieve(e) => {
                    log_error(&e);
                    Task::none()
                }
            },
            Message::MakefileChanged => {
                Task::future(parse_tasks(makefile(&self.base.root_dir), exec_vs_code))
                    .map(MakefileTasksUpdate::from_parse_result)
                    .map(Message::MakefileTasksChanged)
            }
            Message::SettingsChanged(update) => self.update_state(update),
            Message::Cmd(cmd) => self.process_cmd(cmd),
        }
    }

    fn update_state(&mut self, update: SettingsUpdate) -> Task<Message> {
        let Settings {
            pinned_makefile_tasks,
            task_filter,
            category_filters,
        } = &mut self.settings;

        let msg_task = match update {
            SettingsUpdate::AddPinned(task) => {
                if !pinned_makefile_tasks.contains(&task) {
                    pinned_makefile_tasks.push(task);
                    self.update_ui_pinned();
                    None
                } else {
                    Some(
                        Task::future(showInformationMessage(
                            format!("Task '{}' is already pinned.", task.name),
                            Vec::new(),
                        ))
                        .discard(),
                    )
                }
            }
            SettingsUpdate::RemovePinned(idx) => {
                if idx < pinned_makefile_tasks.len() {
                    pinned_makefile_tasks.remove(idx);
                    self.update_ui_pinned();
                    None
                } else {
                    Some(
                        Task::future(showInformationMessage(
                            format!("Task no. '{idx}' could not be unpinned."),
                            Vec::new(),
                        ))
                        .discard(),
                    )
                }
            }
            SettingsUpdate::TaskFilter(tf) => {
                *task_filter = tf;
                self.update_ui();
                None
            }
            SettingsUpdate::CategoryFilter(cf) => {
                *category_filters = cf;
                self.update_ui();
                None
            }
        };

        let tasks = iter::once(
            Task::future(persist_state_vs_code(
                settings_key(&self.base.root_dir),
                self.settings.clone(),
            ))
            .discard(),
        )
        .chain(msg_task);

        Task::batch(tasks)
    }

    fn update_ui(&self) {
        let Settings {
            task_filter,
            category_filters,
            ..
        } = &self.settings;

        let tasks = self.makefile_tasks.filtered(task_filter, category_filters);
        self.ui.update(CargoMakeTreeProviderHandler::new(tasks));
    }

    fn update_ui_pinned(&self) {
        let tasks = self.settings.pinned_makefile_tasks.clone();
        self.ui_pinned
            .update(CargoMakePinnedTreeProviderHandler::new(tasks));
    }
}

fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.cargo_make.ui_settings")
}

fn makefile(root_dir: &str) -> String {
    format!("{root_dir}/Makefile.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pinned_makefile_tasks: MakefileTasks,
    task_filter: String,
    category_filters: Vec<String>,
}
