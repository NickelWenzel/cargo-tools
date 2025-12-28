pub use super::selection::Update;

use crate::app::cargo::{
    command::{Explicit, Implicit},
    metadata::MetadataUpdate,
    selection::{self},
};
use iced_headless::Subscription;

pub enum Message {
    Update(Update),
    Task(Task),
}

pub enum Task {
    ImplicitCommand(Implicit),
    ExplicitCommand(Explicit),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub selection: selection::State,
}

use serde::{Deserialize, Serialize};
use Message as Msg;

pub trait Ui {
    fn update(&mut self, update: MetadataUpdate);

    fn subscription(&self) -> Subscription<Msg>;
}
