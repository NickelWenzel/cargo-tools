use std::fmt::Debug;

use iced_headless::Subscription;
use serde::{Deserialize, Serialize};

use crate::app::cargo_make::tasks::{MakefileTasks, MakefileTasksUpdate};

#[derive(Debug, Clone)]
pub enum Message<CustomUpdate: Clone> {
    MakefileTasks(MakefileTasksUpdate),
    Task(Task),
    Custom(CustomUpdate),
    RootDirUpdate(String),
}

#[derive(Debug, Clone)]
pub struct Task(String);

impl Task {
    pub fn into_name(self) -> String {
        self.0
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
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
