use std::fmt::Debug;

pub use super::selection;

use crate::app::cargo::metadata::MetadataUpdate;
use iced_headless::Subscription;

#[derive(Debug, Clone)]
pub enum Message<CustomUpdate: Clone> {
    Selection(selection::Update),
    Metadata(MetadataUpdate),
    Custom(CustomUpdate),
    RootDirUpdate(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub selection: selection::State,
}

use Message as Msg;
use serde::{Deserialize, Serialize};

pub trait Ui: Sized {
    type CustomUpdate: Debug + Clone + Send;

    fn update(
        &mut self,
        msg: Msg<Self::CustomUpdate>,
    ) -> iced_headless::Task<Msg<Self::CustomUpdate>>;

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>>;
}
