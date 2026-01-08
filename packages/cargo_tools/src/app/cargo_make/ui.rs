use std::fmt::Debug;

use iced_headless::Subscription;
use serde::{Deserialize, Serialize};

use crate::app::cargo_make::tasks::{MakefileTask, MakefileTasks, MakefileTasksUpdate};

#[derive(Debug, Clone)]
pub enum Message<CustomUpdate: Clone> {
    Update(Update),
    MakefileTasks(MakefileTasksUpdate),
    Task(Task),
    Custom(CustomUpdate),
}

#[derive(Debug, Clone)]
pub enum Update {
    AddPinned(MakefileTask),
    RemovePinned(usize),
}

#[derive(Debug, Clone)]
pub enum Task {
    MakeTask(String),
    Pinned(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub pinned: MakefileTasks,
}

use Message as Msg;

pub trait Ui: Sized {
    type CustomUpdate: Debug + Clone;

    fn update(
        &mut self,
        update: Msg<Self::CustomUpdate>,
    ) -> iced_headless::Task<Msg<Self::CustomUpdate>>;

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>>;
}
