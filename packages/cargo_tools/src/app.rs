pub mod cargo_make;
pub mod cargo_settings;
pub mod state;

use iced_headless::{Subscription, Task};

use crate::{
    app::{
        cargo_make::{CargoMake, CargoMakeMessage, CargoMakeUi},
        cargo_settings::{CargoSettings, CargoSettingsMessage},
    },
    context::Context,
    runtime::Runtime,
};

pub enum AppMessage {
    MetadataHandler(CargoSettingsMessage),
    MakefileHandler(CargoMakeMessage),
}

use AppMessage as Msg;

pub trait AppServices {
    type RuntimeT: Runtime;
    type ContextT: Context;
    type CargoMakeUiT: CargoMakeUi;
}

pub struct App {
    metadata_handler: CargoSettings,
    makefile_handler: CargoMake,
}

impl App {
    pub fn update<AppServicesT: AppServices>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::MetadataHandler(msg) => self
                .metadata_handler
                .update::<AppServicesT::RuntimeT>(msg)
                .map(Msg::MetadataHandler),
            Msg::MakefileHandler(msg) => self
                .makefile_handler
                .update::<AppServicesT::RuntimeT, AppServicesT::CargoMakeUiT>(msg)
                .map(Msg::MakefileHandler),
        }
    }

    pub fn subscription<AppServicesT: AppServices>(&self) -> Subscription<Msg> {
        let config_sub = self
            .metadata_handler
            .subscription::<AppServicesT::RuntimeT>()
            .map(Msg::MetadataHandler);

        let makefile_sub = self
            .makefile_handler
            .subscription::<AppServicesT::RuntimeT, AppServicesT::ContextT>()
            .map(Msg::MakefileHandler);

        Subscription::batch([config_sub, makefile_sub])
    }
}
