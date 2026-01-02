use cargo_tools::app::cargo;
use iced_headless::Subscription;

pub struct Ui;

impl cargo::ui::Ui for Ui {
    fn update(&mut self, update: cargo::metadata::MetadataUpdate) {
        todo!()
    }

    fn subscription(&self) -> Subscription<cargo::ui::Message> {
        todo!()
    }
}
