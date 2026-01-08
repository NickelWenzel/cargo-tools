use std::{
    iter,
    sync::{Arc, Mutex},
};

use async_broadcast::{Receiver, broadcast};
use cargo_metadata::Metadata;
use cargo_tools::{
    app::cargo::{
        self,
        selection::{self},
    },
    profile::Profile,
};
use iced_headless::{Subscription, Task};

use cargo::ui::Message as Msg;

use crate::{
    app::StaticHashStream,
    command::{Command, register_cargo_commands},
};

#[derive(Debug)]
pub struct Ui {
    cmd_data: CommandData,
    rx: Receiver<Msg<()>>,
    cmds: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct CommandData {
    pub metadata: Arc<Mutex<UiMetadata>>,
    pub selection: Arc<Mutex<selection::State>>,
}

impl Ui {
    pub fn new(selection: selection::State) -> Self {
        let (tx, rx) = broadcast(100);
        let data = CommandData {
            metadata: Arc::new(Mutex::new(UiMetadata::default())),
            selection: Arc::new(Mutex::new(selection)),
        };

        Self {
            cmd_data: data.clone(),
            rx,
            cmds: register_cargo_commands(tx, data),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiMetadata {
    pub metadata: Option<Metadata>,
    pub profiles: Vec<Profile>,
}

impl UiMetadata {
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn package_options(&self) -> Vec<Option<String>> {
        let Some(metadata) = &self.metadata else {
            return Vec::new();
        };
        iter::once(None)
            .chain(
                metadata
                    .workspace_packages()
                    .iter()
                    .map(|p| Some(p.name.to_string())),
            )
            .collect()
    }
}

impl cargo::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, update: Msg<Self::CustomUpdate>) -> Task<Msg<Self::CustomUpdate>> {
        match update {
            Msg::Selection(update) => self.cmd_data.selection.lock().unwrap().update(update),
            Msg::Metadata(update) => match update {
                cargo::metadata::MetadataUpdate::Metadata(metadata) => {
                    self.cmd_data.metadata.lock().unwrap().metadata = Some(metadata)
                }
                cargo::metadata::MetadataUpdate::Profiles(profiles) => {
                    self.cmd_data.metadata.lock().unwrap().profiles = profiles
                }
                cargo::metadata::MetadataUpdate::NoCargoToml => {
                    *self.cmd_data.metadata.lock().unwrap() = UiMetadata::default()
                }
                cargo::metadata::MetadataUpdate::FailedToRetrieve => {}
            },
            Msg::Task(_) => {}
            Msg::Custom(_) => {}
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>> {
        let stream = StaticHashStream::new(self.rx.clone(), "vscode_cargo");
        Subscription::run_with(stream, |stream| stream.clone())
    }
}
