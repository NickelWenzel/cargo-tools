use cargo_tools::app::cargo;
use iced_headless::{Subscription, Task};

#[derive(Debug)]
pub struct Ui;

impl cargo::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, update: cargo::ui::Message<Self>) -> Task<cargo::ui::Message<Ui>> {
        todo!()
    }

    fn subscription(&self) -> Subscription<cargo::ui::Message<Self>> {
        todo!()
    }
}
