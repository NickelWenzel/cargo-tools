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
    runtime::Runtime,
};

pub enum AppMessage {
    MetadataHandler(CargoMessage),
    MakefileHandler(CargoMakeMessage),
}

use crate::app::cargo_make::ui::Ui as CargoMakeUi;
use AppMessage as Msg;

pub trait AppServices {
    type RuntimeT: Runtime;
}

pub struct App<CargoMakeUiT: CargoMakeUi> {
    metadata_handler: Cargo,
    makefile_handler: CargoMake<CargoMakeUiT>,
}

impl<CargoMakeUiT: CargoMakeUi> App<CargoMakeUiT> {
    pub fn update<Services: AppServices>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::MetadataHandler(msg) => self
                .metadata_handler
                .update::<Services::RuntimeT>(msg)
                .map(Msg::MetadataHandler),
            Msg::MakefileHandler(msg) => self
                .makefile_handler
                .update::<Services::RuntimeT>(msg)
                .map(Msg::MakefileHandler),
        }
    }

    pub fn subscription<Services: AppServices>(&self) -> Subscription<Msg> {
        self.makefile_handler
            .subscription::<Services::RuntimeT>()
            .map(Msg::MakefileHandler)
    }
}
