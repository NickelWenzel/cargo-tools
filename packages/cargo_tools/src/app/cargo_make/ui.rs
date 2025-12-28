use iced_headless::Subscription;
use serde::{Deserialize, Serialize};

use crate::app::cargo_make::tasks::{MakefileTask, MakefileTasks, MakefileTasksUpdate};

pub enum Message {
    Update(Update),
    Task(Task),
}

pub enum Update {
    AddPinned(MakefileTask),
    RemovePinned(usize),
}

pub enum Task {
    MakeTask(String),
    Pinned(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub pinned: MakefileTasks,
}

use Message as Msg;

pub trait Ui {
    fn update(&mut self, update: MakefileTasksUpdate);

    fn subscription(&self) -> Subscription<Msg>;
}
