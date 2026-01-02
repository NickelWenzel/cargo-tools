use cargo_tools::app::cargo_make;
use iced_headless::Subscription;

pub struct Ui;

impl cargo_make::ui::Ui for Ui {
    fn update(&mut self, update: cargo_make::tasks::MakefileTasksUpdate) {
        todo!()
    }

    fn subscription(&self) -> Subscription<cargo_make::ui::Message> {
        todo!()
    }
}
