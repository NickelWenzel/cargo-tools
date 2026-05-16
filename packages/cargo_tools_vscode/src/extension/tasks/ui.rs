use std::iter;

use iced_viewless::Task;

use crate::extension::tasks::{
    cargo_make,
    pinned::{self, SettingsUpdate},
};

#[derive(Debug)]
pub enum Message {
    CargoMake(cargo_make::Message),
    Pinned(pinned::Message),
}

pub struct Tasks {
    cargo_make: cargo_make::Ui,
    pinned: pinned::Ui,
}

impl Tasks {
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        let (cargo_make, cargo_make_task) = cargo_make::Ui::init(root_dir.clone());
        let (pinned, pinned_task) = pinned::Ui::init(root_dir.clone());

        let this = Self { cargo_make, pinned };
        let task = Task::batch([
            cargo_make_task.map(Message::CargoMake),
            pinned_task.map(Message::Pinned),
        ]);

        (this, task)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::CargoMake(msg) => {
                let (task, event) = self.cargo_make.update(msg);
                Task::batch(
                    iter::once(task.map(Message::CargoMake))
                        .chain(event.map(|evt| Task::done(evt.into_message()))),
                )
            }
            Message::Pinned(msg) => self
                .pinned
                .update(self.cargo_make.makefile_tasks(), msg)
                .map(Message::Pinned),
        }
    }
}

trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for cargo_make::Event {
    fn into_message(self) -> Message {
        match self {
            cargo_make::Event::AddPinned(task) => Message::Pinned(
                pinned::Message::SettingsChanged(SettingsUpdate::AddPinned(task)),
            ),
        }
    }
}
