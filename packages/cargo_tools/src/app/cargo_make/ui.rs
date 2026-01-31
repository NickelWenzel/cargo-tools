use std::fmt::Debug;

use iced_headless::Subscription;
use serde::{Deserialize, Serialize};

use crate::{
    app::cargo_make::tasks::{MakefileTasks, MakefileTasksUpdate},
    configuration::{Configuration, Context},
    runtime::{CargoTask, Task},
};

#[derive(Debug, Clone)]
pub enum Message<CustomUpdate: Clone> {
    MakefileTasks(MakefileTasksUpdate),
    Task(Maketask),
    Custom(CustomUpdate),
    RootDirUpdate(String),
}

#[derive(Debug, Clone)]
pub struct Maketask(String);

impl Maketask {
    pub fn into_name(self) -> String {
        self.0
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn into_task(self, config: &impl Configuration) -> CargoTask {
        let ctx = Context::General;
        let config_cmd = config.get_cargo_command(ctx);
        let mut cmd = config_cmd.split_whitespace().map(String::from);
        let (cmd, mut args) = (cmd.next().unwrap(), cmd.collect::<Vec<_>>());
        args.extend(["make".to_string(), self.into_name()]);

        CargoTask::CargoMake(Task {
            cmd,
            args,
            env: config.get_env(ctx),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub pinned: MakefileTasks,
}

use Message as Msg;

pub trait Ui: Sized {
    type CustomUpdate: Debug + Clone + Send;

    fn update(
        &mut self,
        update: Msg<Self::CustomUpdate>,
    ) -> iced_headless::Task<Msg<Self::CustomUpdate>>;

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>>;
}
