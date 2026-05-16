use std::{iter, ops::Deref};

use cargo_tools::cargo_make::{MakefileTask, MakefileTasks};
use futures::channel::mpsc::channel;
use iced_viewless::Task;

use serde::{Deserialize, Serialize};

use crate::{
    environment::makefile_task_context,
    extension::{
        CommandBinding,
        tasks::pinned::{
            command::{Command, register_pinned_commands},
            tree_provider::CargoMakePinnedTreeProviderHandler,
        },
    },
    quick_pick::SelectInput,
    runtime::{CHANNEL_CAPACITY, VsCodeTask, get_state_vs_code, persist_state_vs_code},
    vs_code_api::{CargoMakePinnedTreeProvider, execute_task, log_error, showInformationMessage},
};

#[derive(Debug, Clone)]
pub enum Message {
    SettingsChanged(SettingsUpdate),
    Cmd(Command),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    AddPinned(MakefileTask),
    RemovePinned(usize),
}

#[derive(Debug)]
pub struct Ui {
    settings: Settings,
    ui: CargoMakePinnedTreeProvider,
    _cmds: Vec<CommandBinding>,
    root_dir: String,
}

impl Ui {
    /// Inits all data and update channels
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_pinned_commands(cmd_tx);

        let settings: Settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let handler =
            CargoMakePinnedTreeProviderHandler::new(settings.pinned_makefile_tasks.clone());

        let this = Self {
            settings,
            ui: CargoMakePinnedTreeProvider::new(handler),
            _cmds,
            root_dir,
        };

        // makefile update and cmd will run for the lifetime of the extension
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);

        (this, cmd)
    }

    pub fn update(&mut self, makefile_tasks: &MakefileTasks, msg: Message) -> Task<Message> {
        match msg {
            Message::SettingsChanged(update) => self.update_state(update),
            Message::Cmd(cmd) => self.handle_cmd(makefile_tasks, cmd),
        }
    }

    fn update_state(&mut self, update: SettingsUpdate) -> Task<Message> {
        let Settings {
            pinned_makefile_tasks,
        } = &mut self.settings;

        let msg_task = match update {
            SettingsUpdate::AddPinned(task) => {
                if !pinned_makefile_tasks.contains(&task) {
                    pinned_makefile_tasks.push(task);
                    self.update_ui();
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
                    self.update_ui();
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
        };

        let tasks = iter::once(
            Task::future(persist_state_vs_code(
                settings_key(&self.root_dir),
                self.settings.clone(),
            ))
            .discard(),
        )
        .chain(msg_task);

        Task::batch(tasks)
    }

    fn update_ui(&self) {
        let tasks = self.settings.pinned_makefile_tasks.clone();
        self.ui
            .update(CargoMakePinnedTreeProviderHandler::new(tasks));
    }

    fn handle_cmd(&self, makefile_tasks: &MakefileTasks, cmd: Command) -> Task<Message> {
        match cmd {
            Command::Add => {
                let input = SelectInput {
                    options: makefile_tasks.deref().clone(),
                    current: Vec::new(),
                };
                done(async move { input.select().await.map(SettingsUpdate::AddPinned) })
            }
            Command::Remove(task) => {
                let Some(idx) = self
                    .settings
                    .pinned_makefile_tasks
                    .iter()
                    .position(|pinned| pinned.name == task)
                else {
                    return Task::none();
                };
                Task::done(SettingsUpdate::RemovePinned(idx).into_cargo_make_msg())
            }
            Command::Execute(task) => self.make_task_exec(task),
            Command::Execute1 => self.execute_pinned(0),
            Command::Execute2 => self.execute_pinned(1),
            Command::Execute3 => self.execute_pinned(2),
            Command::Execute4 => self.execute_pinned(3),
            Command::Execute5 => self.execute_pinned(4),
        }
    }

    fn execute_pinned(&self, idx: usize) -> Task<Message> {
        match self.settings.pinned_makefile_tasks.get(idx) {
            Some(task) => self.make_task_exec(task.name.clone()),
            None => Task::future(showInformationMessage(
                format!("There is no task no. {} pinned ", idx + 1),
                Vec::new(),
            ))
            .discard(),
        }
    }

    fn make_task_exec(&self, make_task: String) -> Task<Message> {
        match MakefileTask::try_into_process(make_task, makefile_task_context()) {
            Ok(process) => Task::future(execute_task(VsCodeTask::cargo_make(process))).discard(),
            Err(e) => {
                log_error(&e.to_string());
                Task::none()
            }
        }
    }
}

trait IntoCargoMakeMessage {
    fn into_cargo_make_msg(self) -> Message;
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
    format!("{root_dir}.cargo_tools.tasks.pinned.ui_settings")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pinned_makefile_tasks: MakefileTasks,
}
