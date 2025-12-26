pub mod cargo;
pub mod cargo_make;
pub mod command;
pub mod selection;

use iced_headless::{Subscription, Task};

use crate::{
    app::{
        cargo::{Cargo, CargoMessage},
        cargo_make::{CargoMake, CargoMakeMessage},
    },
    context::Context,
    runtime::Runtime,
};

pub enum AppMessage {
    MetadataHandler(CargoMessage),
    MakefileHandler(CargoMakeMessage),
}

use AppMessage as Msg;

pub trait AppServices {
    type RuntimeT: Runtime;
    type ContextT: Context;
}

pub struct App {
    metadata_handler: Cargo,
    makefile_handler: CargoMake,
}

impl App {
    pub fn update<Services: AppServices>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::MetadataHandler(msg) => self
                .metadata_handler
                .update::<Services::RuntimeT, Services::ContextT>(msg)
                .map(Msg::MetadataHandler),
            Msg::MakefileHandler(msg) => self
                .makefile_handler
                .update::<Services::RuntimeT, Services::ContextT>(msg)
                .map(Msg::MakefileHandler),
        }
    }

    pub fn subscription<Services: AppServices>(&self) -> Subscription<Msg> {
        self.makefile_handler
            .subscription::<Services::RuntimeT, Services::ContextT>()
            .map(Msg::MakefileHandler)
    }
}
