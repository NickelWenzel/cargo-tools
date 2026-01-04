use std::sync::{Arc, Mutex};

use cargo_metadata::Metadata;
use cargo_tools::{
    app::cargo::{self, selection},
    profile::Profile,
};
use iced_headless::{Subscription, Task};

use cargo::ui::Message as Msg;

#[derive(Debug, Clone)]
pub struct Ui {
    pub data: Arc<Mutex<Data>>,
    pub selection: Arc<Mutex<selection::State>>,
}

impl Ui {
    pub fn new() -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    metadata: Metadata,
    profiles: Vec<Profile>,
}

impl Data {
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }
}

impl cargo::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, _update: Msg<Self>) -> Task<Msg<Self>> {
        todo!()
    }

    fn subscription(&self) -> Subscription<Msg<Self>> {
        Subscription::run(|| super::MSG_RX.lock().unwrap().clone()).filter_map(|msg| match msg {
            super::Message::Cargo(msg) => Some(msg),
            super::Message::CargoMake(_) => None,
        })
    }
}
