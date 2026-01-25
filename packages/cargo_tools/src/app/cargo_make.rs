pub mod tasks;
pub mod ui;

use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{
    app::cargo_make::tasks::{MakefileTasksUpdate, parse_tasks},
    configuration::{self, Configuration},
    runtime::{self, CargoTask, Runtime},
};

#[derive(Debug, Clone)]
pub enum CargoMakeMessage<Ui: ui::Ui> {
    RootDirUpdate(String),
    MakefileUpdate,
    MakefileTasksUpdate(MakefileTasksUpdate),
    Ui(ui::Message<Ui::CustomUpdate>),
}

use CargoMakeMessage as Msg;

#[derive(Debug, Default)]
pub struct CargoMake<Ui: ui::Ui + Default> {
    root_dir: String,
    ui: Ui,
    state: ui::State,
}

impl<Ui: ui::Ui + Default + 'static> CargoMake<Ui> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg<Ui>) -> Task<Msg<Ui>> {
        RT::log("Cargo make update received".to_string());
        match msg {
            Msg::RootDirUpdate(root_dir) => {
                RT::log(format!(
                    "Cargo make update received: new root dir {root_dir}"
                ));
                self.update_root_dir::<RT>(root_dir)
            }
            Msg::MakefileUpdate => {
                RT::log("Cargo make update received: makefile updated".to_string());
                Task::future(parse_tasks::<RT>(self.makefile())).map(Msg::MakefileTasksUpdate)
            }
            Msg::MakefileTasksUpdate(tasks_update) => {
                RT::log("Cargo make update received: makefile tasks updated".to_string());
                self.update_tasks::<RT>(tasks_update)
            }
            Msg::Ui(msg) => {
                let task = match &msg {
                    ui::Message::Task(task) => self.exec_task::<RT>(task.clone()),
                    ui::Message::MakefileTasks(_)
                    | ui::Message::Custom(_)
                    | ui::Message::RootDirUpdate(_) => Task::none(),
                };
                let ui = self.ui.update(msg).map(Msg::Ui);

                Task::batch([task, ui])
            }
        }
    }

    fn update_root_dir<RT: Runtime>(&mut self, root_dir: String) -> Task<Msg<Ui>> {
        self.root_dir = root_dir;

        if let Some(s) = RT::get_state(self.state_key()) {
            self.state = s;
        }

        Task::future(parse_tasks::<RT>(self.makefile())).map(Msg::MakefileTasksUpdate)
    }

    fn exec_task<RT: Runtime>(&self, task: ui::Task) -> Task<Msg<Ui>> {
        let (cmd, mut args, env) = {
            let config = RT::get_configuration();
            let ctx = configuration::Context::General;
            let config_cmd = config.get_cargo_command(ctx);
            let mut cmd = config_cmd.split_whitespace().map(String::from);
            let (cmd, args) = (cmd.next().unwrap(), cmd.collect::<Vec<_>>());
            (cmd, args, config.get_env(ctx))
        };

        args.extend(["make".to_string(), task.into_name()]);

        Task::future(RT::exec_task(CargoTask::CargoMake(runtime::Task {
            cmd,
            args,
            env,
        })))
        .discard()
    }

    fn update_tasks<RT: Runtime>(&mut self, tasks_update: MakefileTasksUpdate) -> Task<Msg<Ui>> {
        let makefile = self.makefile();
        let file_change = Task::future(async move {
            let ret = RT::file_changed_notifier(makefile).next().await;
            RT::log(format!("Makefile.toml changed: {ret:?}"));
            ret
        })
        .and_then(|()| Task::done(Msg::MakefileUpdate));
        let ui = Task::done(ui::Message::MakefileTasks(tasks_update)).map(Msg::Ui);

        Task::batch([file_change, ui])
    }

    pub fn subscription<RT: Runtime>(&self) -> Subscription<Msg<Ui>> {
        let root = Subscription::run(RT::current_dir_notitifier).map(Msg::RootDirUpdate);
        let ui = self.ui.subscription().map(Msg::Ui);

        Subscription::batch([root, ui])
    }

    pub fn state_key(&self) -> String {
        format!("{}.cargo_tools.cargo_make.state", self.root_dir)
    }

    pub fn makefile(&self) -> String {
        format!("{}/Makefile.toml", self.root_dir)
    }
}
