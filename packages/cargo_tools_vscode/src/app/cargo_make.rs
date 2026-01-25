pub mod command;

use cargo_tools::{
    app::cargo_make::{
        self,
        tasks::{MakefileTask, MakefileTasks, MakefileTasksUpdate},
    },
    runtime::Runtime,
};
use futures::{
    SinkExt, Stream, StreamExt,
    channel::mpsc::{Sender, channel},
};
use iced_headless::{Subscription, Task, stream};

use cargo_make::ui::Message as Msg;
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        CargoMakeMsg, VsCodeTask,
        cargo_make::command::{Command, register::register_cargo_make_commands},
    },
    runtime::{CHANNEL_CAPACITY, VsCodeRuntime},
    vs_code_api::log,
};

#[derive(Debug, Clone)]
pub enum UiMessage {
    CmdTx(Sender<Command>),
    Cmd(Command),
    Settings(SettingsUpdate),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    AddPinned(MakefileTask),
    RemovePinned(usize),
    TaskFilter(String),
    CategoryFilter(Vec<String>),
}

#[derive(Debug, Default)]
pub struct Ui {
    makefile_tasks: MakefileTasks,
    settings: Settings,
    cmds: Vec<VsCodeTask>,
    root_dir: String,
}

impl Ui {
    fn update_state(&mut self, update: SettingsUpdate) -> Task<CargoMakeMsg> {
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
        Task::future(VsCodeRuntime::persist_state(
            self.settings_key(),
            self.settings.clone(),
        ))
        .discard()
    }

    pub fn settings_key(&self) -> String {
        format!("{}.cargo_tools.cargo.ui_settings", self.root_dir)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pinned_makefile_tasks: MakefileTasks,
    task_filter: String,
    category_filters: Vec<String>,
}

impl cargo_make::ui::Ui for Ui {
    type CustomUpdate = UiMessage;

    fn update(&mut self, msg: CargoMakeMsg) -> Task<CargoMakeMsg> {
        log("Cargo make Ui update received");
        match msg {
            Msg::MakefileTasks(update) => {
                match update {
                    MakefileTasksUpdate::New(makefile_tasks) => {
                        self.makefile_tasks = makefile_tasks
                    }
                    MakefileTasksUpdate::NoMakefile => self.makefile_tasks = Vec::new(),
                    MakefileTasksUpdate::FailedToRetrieve => {}
                }
                Task::none()
            }
            Msg::Task(_) => Task::none(),
            Msg::Custom(msg) => match msg {
                UiMessage::CmdTx(tx) => {
                    self.cmds = register_cargo_make_commands(tx);
                    Task::none()
                }
                UiMessage::Cmd(cmd) => match cmd {
                    Command::RunTask(_) => todo!(),
                    Command::SelectAndRunTask => todo!(),
                    Command::SelectTaskFilter => todo!(),
                    Command::EditTaskFilter(_) => todo!(),
                    Command::SelectCategoryFilter => todo!(),
                    Command::EditCategoryFilter(_) => todo!(),
                    Command::ClearAllFilters => todo!(),
                    Command::PinTask(makefile_task) => todo!(),
                    Command::Pinned(pinned) => todo!(),
                },
                UiMessage::Settings(update) => self.update_state(update),
            },
            Msg::RootDirUpdate(root_dir) => {
                self.root_dir = root_dir;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<CargoMakeMsg> {
        Subscription::run(command_stream).map(Msg::Custom)
    }
}

fn command_stream() -> impl Stream<Item = UiMessage> {
    stream::channel(CHANNEL_CAPACITY, async |mut out| {
        let (tx, mut rx) = channel(CHANNEL_CAPACITY);
        if let Err(e) = out.send(UiMessage::CmdTx(tx.clone())).await {
            log(&format!(
                "Failed to send cargo ui command sender back to ui: {e:?}"
            ));
        }
        while let Some(msg) = rx.next().await {
            log(&format!("Sending command message to cargo Ui'{msg:?}'"));
            if let Err(e) = out.send(UiMessage::Cmd(msg)).await {
                log(&format!(
                    "Failed to send command message to cargo UI '{e:?}'"
                ));
            }
        }
        log("Cargo Ui command stream closed unexpectedly.");
    })
}
