use cargo_tools::app::cargo;
use iced_headless::{Subscription, Task};

use cargo::ui::Message as Msg;

#[derive(Debug, Clone)]
pub struct Ui;

impl cargo::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, update: Msg<Self>) -> Task<Msg<Self>> {
        todo!()
    }

    fn subscription(&self) -> Subscription<Msg<Self>> {
        Subscription::run(|| super::MSG_RX.lock().unwrap().clone()).filter_map(|msg| match msg {
            super::Message::Cargo(msg) => Some(msg),
            super::Message::CargoMake(_) => None,
        })
    }
}
