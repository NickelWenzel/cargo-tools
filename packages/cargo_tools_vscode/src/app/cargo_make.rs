use cargo_tools::app::cargo_make;
use iced_headless::{Subscription, Task};

#[derive(Debug, Clone)]
pub struct Ui;

impl cargo_make::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(
        &mut self,
        update: cargo_make::ui::Message<Self>,
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
