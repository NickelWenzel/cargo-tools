pub mod command;

use cargo_tools::{
    app::cargo_make::tasks::{MakefileTask, MakefileTasks, MakefileTasksUpdate, parse_tasks},
    runtime::Runtime as _,
};
use futures::channel::mpsc::channel;
use iced_headless::Task;

use serde::{Deserialize, Serialize};

use crate::{
    app::{
        base::{Base, send_file_changed},
        cargo_make::command::{Command, register_cargo_make_commands},
    },
    runtime::{CHANNEL_CAPACITY, VsCodeRuntime as Runtime},
    vs_code_api::{TsFileWatcher, set_makefile_context},
};

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
    base: Base,
}

impl Ui {
    /// Inits all data and update channels
    pub fn new(root_dir: String) -> (Self, Task<Message>) {
        // Init makefile updates
        let (makefile_changed_tx, makefile_changed_rx) = channel(CHANNEL_CAPACITY);
        let file_watcher = TsFileWatcher::new(send_file_changed(makefile_changed_tx));

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let cmds = register_cargo_make_commands(cmd_tx);

        let settings = Runtime::get_state(settings_key(&root_dir)).unwrap_or_default();

        let base = Base {
            cmds,
            file_watcher,
            root_dir,
        };

        let this = Self {
            makefile_tasks: Vec::new(),
            settings,
            base,
        };

        // makefile update and cmd will run for the lifetime of the extension
        let manifest_update = Task::stream(makefile_changed_rx).map(|()| Message::MakefileChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        // makefile_tasks is to initially parse the available makefile tasks
        let makefile_tasks = Task::future(parse_tasks::<Runtime>(makefile(&this.base.root_dir)))
            .map(Message::MakefileTasksChanged);
        let tasks = Task::batch([manifest_update, cmd, makefile_tasks]);

        (this, tasks)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::MakefileTasksChanged(update) => match update {
                MakefileTasksUpdate::New(makefile_tasks) => {
                    self.makefile_tasks = makefile_tasks;
                    Task::future(set_makefile_context(true)).discard()
                }
                MakefileTasksUpdate::NoMakefile => {
                    self.makefile_tasks = Vec::new();
                    Task::future(set_makefile_context(false)).discard()
                }
                // For invalid makefiles leave everything as is
                MakefileTasksUpdate::FailedToRetrieve => Task::none(),
            },
            Message::MakefileChanged => {
                Task::future(parse_tasks::<Runtime>(makefile(&self.base.root_dir)))
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
        match update {
            SettingsUpdate::AddPinned(task) => {
                if pinned_makefile_tasks.contains(&task) {
                    pinned_makefile_tasks.push(task);
                }
            }
            SettingsUpdate::RemovePinned(idx) => {
                if idx < pinned_makefile_tasks.len() {
                    pinned_makefile_tasks.remove(idx);
                }
            }
            SettingsUpdate::TaskFilter(tf) => *task_filter = tf,
            SettingsUpdate::CategoryFilter(cf) => *category_filters = cf,
        };
        Task::future(Runtime::persist_state(
            settings_key(&self.base.root_dir),
            self.settings.clone(),
        ))
        .discard()
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
