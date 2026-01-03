use cargo_tools::app::cargo_make;
use iced_headless::{Subscription, Task};

#[derive(Debug)]
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
        todo!()
    }
}
