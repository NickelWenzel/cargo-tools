pub mod command;

use cargo_tools::app::cargo_make::{
    self,
    tasks::{MakefileTasks, MakefileTasksUpdate},
    ui::Update,
};
use futures::{
    SinkExt, Stream, StreamExt,
    channel::mpsc::{Sender, channel},
};
use iced_headless::{Subscription, Task, stream};

use cargo_make::ui::Message as Msg;

use crate::{
    app::{
        CargoMakeMsg, VsCodeTask,
        cargo_make::command::{Command, register::register_cargo_make_commands},
    },
    runtime::CHANNEL_CAPACITY,
    vs_code_api::log,
};

#[derive(Debug, Clone)]
pub enum UiMessage {
    CmdTx(Sender<Command>),
    Cmd(Command),
}

#[derive(Debug, Default)]
pub struct Ui {
    makefile_tasks: MakefileTasks,
    pinnedmakefile_tasks: MakefileTasks,
    cmds: Vec<VsCodeTask>,
    root_dir: String,
}

impl cargo_make::ui::Ui for Ui {
    type CustomUpdate = UiMessage;

    fn update(&mut self, msg: CargoMakeMsg) -> Task<CargoMakeMsg> {
        log("Cargo make Ui update received");
        match msg {
            Msg::Update(update) => {
                let pinned = &mut self.pinnedmakefile_tasks;
                match update {
                    Update::AddPinned(makefile_task) => {
                        if !pinned.contains(&makefile_task) {
                            pinned.push(makefile_task);
                        }
                    }
                    Update::RemovePinned(idx) => {
                        if idx < pinned.len() {
                            pinned.remove(idx);
                        }
                    }
                }
                Task::none()
            }
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
                UiMessage::Cmd(cmd) => match cmd {},
            },
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
