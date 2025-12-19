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
    MetadataHandler(MetadataHandlerMessage),
    MakefileHandler(MakefileHandlerMessage),
}

use CargoToolsMessage as Msg;

pub struct CargoTools {
    metadata_handler: MetadataHandler,
    makefile_handler: MakefileHandler,
}

impl CargoTools {
    pub fn update<RuntimeT: Runtime>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::MetadataHandler(msg) => self
                .metadata_handler
                .update::<RuntimeT>(msg)
                .map(Msg::MetadataHandler),
            Msg::MakefileHandler(msg) => self
                .makefile_handler
                .update::<RuntimeT>(msg)
                .map(Msg::MakefileHandler),
        }
    }

    pub fn subscription<RuntimeT: Runtime>(&self) -> Subscription<Msg> {
        let config_sub = self
            .metadata_handler
            .subscription::<RuntimeT>()
            .map(Msg::MetadataHandler);

        let makefile_sub = self
            .makefile_handler
            .subscription::<RuntimeT>()
            .map(Msg::MakefileHandler);

        Subscription::batch([config_sub, makefile_sub])
    }
}
