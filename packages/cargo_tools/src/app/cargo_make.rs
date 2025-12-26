pub mod tasks;
pub mod ui;

use std::collections::HashMap;

use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{
    app::cargo_make::tasks::{parse_tasks, MakefileTasksUpdate},
    runtime::{self, CargoTask, Runtime},
};

pub enum CargoMakeMessage {
    RootDirUpdate(String),
    MakefileUpdate,
    MakefileTasksUpdate(MakefileTasksUpdate),
    Ui(ui::Message),
}

use CargoMakeMessage as Msg;

pub struct CargoMake<Ui: ui::Ui> {
    makefile: String,
    ui: Ui,
    state: ui::State,
}

impl<Ui: ui::Ui> CargoMake<Ui> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT>(root_dir),
            Msg::MakefileUpdate => {
                Task::future(parse_tasks::<RT>(self.makefile.clone())).map(Msg::MakefileTasksUpdate)
            }
            Msg::MakefileTasksUpdate(tasks_update) => self.update_tasks::<RT>(tasks_update),
            Msg::Ui(msg) => match msg {
                ui::Message::Update(update) => self.update_state(update),
                ui::Message::Task(task) => self.exec_task::<RT>(task),
            },
        }
    }

    fn update_root_dir<RT: Runtime>(&mut self, root_dir: String) -> Task<Msg> {
        self.makefile = format!("{root_dir}/Makefile.toml");

        Task::future(parse_tasks::<RT>(self.makefile.clone())).map(Msg::MakefileTasksUpdate)
    }

    fn update_state(&mut self, update: ui::Update) -> Task<Msg> {
        match update {
            ui::Update::AddPinned(task) => {
                if self.state.pinned.contains(&task) {
                    self.state.pinned.push(task);
                }
            }
            ui::Update::RemovePinned(idx) => {
                if idx < self.state.pinned.len() {
                    self.state.pinned.remove(idx);
                }
            }
        };
        Task::none()
    }

    fn exec_task<RT: Runtime>(&self, task: ui::Task) -> Task<Msg> {
        match task {
            ui::Task::MakeTask(name) => {
                Task::future(RT::exec_task(CargoTask::CargoMake(runtime::Task {
                    cmd: "cargo".to_string(),
                    args: vec!["make".to_string(), name],
                    env: HashMap::new(),
                })))
                .discard()
            }
            ui::Task::Pinned(idx) => match self.state.pinned.get(idx) {
                Some(task) => Task::done(task.name.clone())
                    .map(|name| Msg::Ui(ui::Message::Task(ui::Task::MakeTask(name)))),
                None => Task::none(),
            },
        }
    }

    fn update_tasks<RT: Runtime>(&mut self, tasks_update: MakefileTasksUpdate) -> Task<Msg> {
        self.ui.update(tasks_update);

        let makefile = self.makefile.clone();
        Task::future(async move {
            RT::file_changed_notifier(makefile).next().await;
        })
        .map(|()| Msg::MakefileUpdate)
    }

    pub fn subscription<RT: Runtime>(&self) -> Subscription<Msg> {
        let root = Subscription::run(RT::current_dir_notitifier).map(Msg::RootDirUpdate);
        let ui = self.ui.subscription().map(Msg::Ui);

        Subscription::batch([root, ui])
    }
}
