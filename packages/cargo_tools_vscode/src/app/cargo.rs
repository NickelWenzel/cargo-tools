use std::{
    iter,
    sync::{Arc, Mutex},
};

use async_broadcast::{Sender, broadcast};
use cargo_metadata::Metadata;
use cargo_tools::{
    app::cargo::{
        self,
        command::{BuildSubTarget, RunSubTarget},
        selection::{self, Features},
    },
    profile::Profile,
};
use futures::{SinkExt, Stream, StreamExt};
use iced_headless::{Subscription, Task, stream};

use cargo::ui::Message as Msg;

use crate::{
    app::{CargoMsg, IntoCargoMessage, SendResult},
    command::{Command, SelectInput, register_cargo_commands},
    runtime::CHANNEL_CAPACITY,
    vs_code_api::log,
};

#[derive(Debug, Clone)]
pub enum UiUpdate {
    CmdTx(Sender<CargoMsg>),
}

#[derive(Debug)]
pub struct Ui {
    cmd_data: Arc<Mutex<CommandData>>,
    _cmds: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct CommandData {
    pub metadata: UiMetadata,
    pub selection: selection::State,
    pub tx: Sender<CargoMsg>,
}

impl CommandData {
    pub fn profiles(&self) -> SelectInput<Profile> {
        let options = self.metadata.profiles().to_vec();
        let current = vec![self.selection.profile.clone()];

        SelectInput { options, current }
    }

    pub fn packages(&self) -> SelectInput<Option<String>> {
        let options = self.metadata.package_options();
        let current = vec![self.selection.package.clone()];

        SelectInput { options, current }
    }

    pub fn build_target_options(&self) -> Option<SelectInput<Option<BuildSubTarget>>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let selection = &self.selection;

        let options = selection.build_target_options(metadata);
        let current = vec![
            selection
                .package_selection()
                .and_then(|s| s.build_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    pub fn run_target_options(&self) -> Option<SelectInput<Option<RunSubTarget>>> {
        let metadata = self.metadata.metadata.as_ref()?;

        let selection = &self.selection;

        let options = selection.run_target_options(metadata);
        let current = vec![
            selection
                .package_selection()
                .and_then(|s| s.run_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    pub fn bench_target_options(&self) -> Option<SelectInput<Option<String>>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let selection = &self.selection;

        let options = selection.bench_target_options(metadata);
        let current = vec![
            selection
                .package_selection()
                .and_then(|s| s.benchmark_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    pub fn feature_options(&self) -> Option<SelectInput<String>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let selection = &self.selection;

        let options = iter::once("All features".to_string())
            .chain(selection.feature_options(metadata))
            .collect::<Vec<_>>();
        let current = match selection.selected_features() {
            Features::All => ["All features".to_string()].to_vec(),
            Features::Some(features) => features,
        };

        Some(SelectInput { options, current })
    }

    pub async fn send<T: IntoCargoMessage>(&self, msg: T) -> SendResult<CargoMsg> {
        self.tx.broadcast(msg.into_cargo_msg()).await
    }
}

impl Ui {
    pub fn new(selection: selection::State) -> Self {
        let (tx, _) = broadcast(100); // Dummy sender until subscription is ready
        let data = Arc::new(Mutex::new(CommandData {
            metadata: UiMetadata::default(),
            selection,
            tx,
        }));

        Self {
            cmd_data: data.clone(),
            _cmds: register_cargo_commands(data),
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
    type CustomUpdate = UiUpdate;

    fn update(&mut self, msg: Msg<Self::CustomUpdate>) -> Task<Msg<Self::CustomUpdate>> {
        log("Cargo Ui update received");
        let mut data = self.cmd_data.lock().unwrap();
        match msg {
            Msg::Selection(update) => {
                log(&format!(
                    "Cargo Ui update received: new selection '{update:?}'"
                ));
                data.selection.update(update)
            }
            Msg::Metadata(update) => match update {
                cargo::metadata::MetadataUpdate::Metadata(metadata) => {
                    log("Cargo Ui update received: new metadata");
                    data.metadata.metadata = Some(metadata)
                }
                cargo::metadata::MetadataUpdate::Profiles(profiles) => {
                    log("Cargo Ui update received: new profiles");
                    data.metadata.profiles = profiles
                }
                cargo::metadata::MetadataUpdate::NoCargoToml => {
                    data.metadata = UiMetadata::default()
                }
                cargo::metadata::MetadataUpdate::FailedToRetrieve => {}
            },
            Msg::Task(_) => {}
            Msg::Custom(msg) => {
                log(&format!("Cargo Ui update received: custom '{msg:?}'"));
                match msg {
                    UiUpdate::CmdTx(tx) => data.tx = tx,
                }
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>> {
        Subscription::run(command_stream)
    }
}

fn command_stream() -> impl Stream<Item = Msg<UiUpdate>> {
    stream::channel(CHANNEL_CAPACITY, async |mut out| {
        log("Sending command message sender to cargo Ui");
        let (tx, mut rx) = broadcast(CHANNEL_CAPACITY);
        if let Err(e) = out.send(Msg::Custom(UiUpdate::CmdTx(tx.clone()))).await {
            log(&format!(
                "Failed to send cargo ui command sender back to ui: {e:?}"
            ));
        }
        while let Some(msg) = rx.next().await {
            log(&format!("Sending command message to cargo Ui'{msg:?}'"));
            if let Err(e) = out.send(msg).await {
                log(&format!(
                    "Failed to send command message to cargo UI '{e:?}'"
                ));
            }
        }
        log("Cargo Ui command stream closed unexpectedly.");
    })
}
