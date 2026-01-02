pub use super::selection::Update;

use crate::app::cargo::{
    command::{Explicit, Implicit},
    metadata::MetadataUpdate,
    selection::{self},
};
use iced_headless::Subscription;

#[derive(Debug, Clone)]
pub enum Message {
    Update(Update),
    Task(Task),
}

#[derive(Debug, Clone)]
pub enum Task {
    ImplicitCommand(Implicit),
    ExplicitCommand(Explicit),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub selection: selection::State,
}

use Message as Msg;
use serde::{Deserialize, Serialize};

pub trait Ui {
    fn update(&mut self, update: MetadataUpdate);

    fn subscription(&self) -> Subscription<Msg>;
}
