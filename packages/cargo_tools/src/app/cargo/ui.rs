use std::fmt::Debug;

pub use super::selection;

use crate::app::cargo::{
    command::{Explicit, Implicit},
    metadata::MetadataUpdate,
};
use iced_headless::Subscription;

#[derive(Debug, Clone)]
pub enum Message<State: Ui> {
    Selection(selection::Update),
    Metadata(MetadataUpdate),
    Task(Task),
    Custom(State::CustomUpdate),
}

#[derive(Debug, Clone)]
pub enum Task {
    ImplicitCommand(Implicit),
    ExplicitCommand(Explicit),
    AddPlatformTarget(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub selection: selection::State,
}

use Message as Msg;
use serde::{Deserialize, Serialize};

pub trait Ui: Sized {
    type CustomUpdate: Debug + Clone;

    fn update(&mut self, msg: Msg<Self>) -> iced_headless::Task<Msg<Self>>;

    fn subscription(&self) -> Subscription<Msg<Self>>;
}
