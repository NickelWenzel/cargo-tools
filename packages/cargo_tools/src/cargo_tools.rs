pub mod makefile_handler;
pub mod metadata_handler;
pub mod state;

use iced_headless::{Subscription, Task};

use crate::{
    cargo_tools::{
        makefile_handler::{MakefileHandler, MakefileHandlerMessage},
        metadata_handler::{MetadataHandler, MetadataHandlerMessage},
    },
    runtime::Runtime,
};

pub enum CargoToolsMessage {
    ConfigurationHandler(MetadataHandlerMessage),
    MakefileHandler(MakefileHandlerMessage),
}

use CargoToolsMessage as Msg;

pub struct CargoTools {
    config_handler: MetadataHandler,
    makefile_handler: MakefileHandler,
}

impl CargoTools {
    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::ConfigurationHandler(msg) => todo!(),
            Msg::MakefileHandler(msg) => todo!(),
        }
    }

    pub fn subscription<RuntimeT: Runtime + 'static>(&self) -> Subscription<Msg> {
        let config_sub = self
            .config_handler
            .subscription::<RuntimeT>()
            .map(Msg::ConfigurationHandler);

        let makefile_sub = self
            .makefile_handler
            .subscription::<RuntimeT>()
            .map(Msg::MakefileHandler);

        Subscription::batch([config_sub, makefile_sub])
    }
}
