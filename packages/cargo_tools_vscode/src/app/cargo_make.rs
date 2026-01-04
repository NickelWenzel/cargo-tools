use std::sync::{Arc, Mutex};

use cargo_tools::app::cargo_make::{self, tasks::MakefileTasks};
use iced_headless::{Subscription, Task};

#[derive(Debug, Clone)]
pub struct Ui {
    makefile_tasks: Arc<Mutex<MakefileTasks>>,
    state: Arc<Mutex<cargo_make::ui::State>>,
}

impl Ui {
    pub fn new() -> Self {
        todo!()
    }
}

impl cargo_make::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(
        &mut self,
        _update: cargo_make::ui::Message<Self>,
    ) -> Task<cargo_make::ui::Message<Self>> {
        todo!()
    }

    fn subscription(&self) -> Subscription<cargo_make::ui::Message<Self>> {
        Subscription::run(|| super::MSG_RX.lock().unwrap().clone()).filter_map(|msg| match msg {
            super::Message::Cargo(_) => None,
            super::Message::CargoMake(msg) => Some(msg),
        })
    }
}
