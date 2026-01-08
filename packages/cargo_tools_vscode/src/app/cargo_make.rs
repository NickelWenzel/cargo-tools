use std::sync::{Arc, Mutex};

use async_broadcast::{Receiver, broadcast};
use cargo_tools::app::cargo_make::{self, tasks::MakefileTasks};
use iced_headless::{Subscription, Task};

use cargo_make::ui::Message as Msg;

use crate::{
    app::StaticHashStream,
    command::{Command, register_cargo_make_commands},
};

#[derive(Debug)]
pub struct Ui {
    cmd_data: CommandData,
    rx: Receiver<Msg<()>>,
    cmds: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct CommandData {
    makefile_tasks: Arc<Mutex<MakefileTasks>>,
    state: Arc<Mutex<cargo_make::ui::State>>,
}

impl Ui {
    pub fn new(state: cargo_make::ui::State) -> Self {
        let (tx, rx) = broadcast(100);
        let data = CommandData {
            makefile_tasks: Arc::new(Mutex::new(MakefileTasks::new())),
            state: Arc::new(Mutex::new(state)),
        };

        Self {
            cmd_data: data.clone(),
            rx,
            cmds: register_cargo_make_commands(tx, data),
        }
    }
}

impl cargo_make::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, update: Msg<Self::CustomUpdate>) -> Task<Msg<Self::CustomUpdate>> {
        match update {
            cargo_make::ui::Message::Update(update) => match update {
                cargo_make::ui::Update::AddPinned(makefile_task) => {
                    let mut state_guard = self.cmd_data.state.lock().unwrap();
                    state_guard.pinned.push(makefile_task);
                }
                cargo_make::ui::Update::RemovePinned(idx) => {
                    let guard = self.cmd_data.state.lock().unwrap();
                    if idx < guard.pinned.len() {
                        self.cmd_data.state.lock().unwrap().pinned.remove(idx);
                    }
                }
            },
            cargo_make::ui::Message::MakefileTasks(update) => match update {
                cargo_make::tasks::MakefileTasksUpdate::New(makefile_tasks) => {
                    *self.cmd_data.makefile_tasks.lock().unwrap() = makefile_tasks;
                }
                cargo_make::tasks::MakefileTasksUpdate::NoMakefile => {
                    *self.cmd_data.makefile_tasks.lock().unwrap() = MakefileTasks::new();
                }
                cargo_make::tasks::MakefileTasksUpdate::FailedToRetrieve => {}
            },
            cargo_make::ui::Message::Task(_) => {}
            cargo_make::ui::Message::Custom(_) => {}
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>> {
        let stream = StaticHashStream::new(self.rx.clone(), "vscode_cargo_make");
        Subscription::run_with(stream, |stream| stream.clone())
    }
}
