use std::sync::{Arc, Mutex};

use cargo_metadata::{Metadata, MetadataCommand};
use futures::StreamExt;
use iced_headless::{Subscription, Task};
use wasm_async_trait::wasm_async_trait;

use crate::{
    app::state::{State, StateUpdate},
    context::Context,
    runtime::Runtime,
};

#[wasm_async_trait]
pub trait CargoSettingsUi {
    async fn update(metadata: Arc<Mutex<MetadataUpdate>>, state: Arc<Mutex<State>>);
}

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Metadata),
    NoCargoToml,
    FailedToRetrieve,
}
pub enum CargoSettingsMessage {
    RootDirUpdate(String),
    StateUpdate(State),
    ManifestUpdate,
    MetadataUpdate(MetadataUpdate),
}

use CargoSettingsMessage as Msg;

pub struct CargoSettings {
    root_manifest: String,
    workspace_manifests: Vec<String>,
    metadata: Arc<Mutex<MetadataUpdate>>,
    state: Arc<Mutex<State>>,
}

impl CargoSettings {
    pub fn update<RT: Runtime, UI: CargoSettingsUi, CTX: Context>(
        &mut self,
        msg: Msg,
    ) -> Task<Msg> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT, CTX>(root_dir),
            Msg::ManifestUpdate => Task::future(parse_metadata::<RT>(self.root_manifest.clone()))
                .map(Msg::MetadataUpdate),
            Msg::MetadataUpdate(update) => self.update_metadata::<RT, UI>(update),
            Msg::StateUpdate(state) => {
                *self.state.lock().unwrap() = state;
                self.update_ui::<UI>()
            }
        }
    }

    fn update_root_dir<RT: Runtime, CTX: Context>(&mut self, root_dir: String) -> Task<Msg> {
        self.root_manifest = format!("{root_dir}/Cargo.toml");
        let update =
            Task::future(parse_metadata::<RT>(self.root_manifest.clone())).map(Msg::MetadataUpdate);
        let prefix = Task::future(async move {
            CTX::update_prefix(root_dir.clone()).await;
        })
        .discard::<Msg>();
        let tick_state = Task::future(CTX::update_state(StateUpdate::Tick)).discard::<Msg>();
        Task::batch([update, prefix, tick_state])
    }

    fn update_metadata<RT: Runtime, UI: CargoSettingsUi>(
        &mut self,
        metadata_update: MetadataUpdate,
    ) -> Task<Msg> {
        match &metadata_update {
            MetadataUpdate::New(metadata) => {
                self.workspace_manifests = workspace_manifests(metadata);
            }
            MetadataUpdate::NoCargoToml => {
                self.workspace_manifests = Vec::new();
            }
            MetadataUpdate::FailedToRetrieve => {}
        }

        *self.metadata.lock().unwrap() = metadata_update;

        let manifests_changed = self.manifests();
        let manifests = Task::future(async move {
            let notifiers = manifests_changed.into_iter().map(RT::file_changed_notifier);
            futures::stream::select_all(notifiers).next().await;
        })
        .map(|()| Msg::ManifestUpdate);

        let ui = self.update_ui::<UI>();

        Task::batch([manifests, ui])
    }

    fn update_ui<UI: CargoSettingsUi>(&self) -> Task<Msg> {
        let (metadata, state) = (self.metadata.clone(), self.state.clone());
        Task::future(UI::update(metadata, state)).discard::<Msg>()
    }

    pub fn subscription<RuntimeT: Runtime, ContextT: Context>(&self) -> Subscription<Msg> {
        let root_dir = Subscription::run(RuntimeT::current_dir_notitifier).map(Msg::RootDirUpdate);
        let state = Subscription::run(ContextT::state_receiver).map(Msg::StateUpdate);

        Subscription::batch([root_dir, state])
    }

    fn manifests(&self) -> Vec<String> {
        let mut manifests = self.workspace_manifests.clone();
        manifests.push(self.root_manifest.clone());
        manifests
    }
}

fn workspace_manifests(metadata: &Metadata) -> Vec<String> {
    metadata
        .workspace_packages()
        .into_iter()
        .map(|p| p.manifest_path.to_string())
        .collect()
}

pub async fn parse_metadata<RuntimeT: Runtime>(manifest_file: String) -> MetadataUpdate {
    // Construct cargo metadata command with manifest path
    let command =
        format!("cargo metadata --format-version 1 --manifest-path {manifest_file} --no-deps");

    // Execute command via runtime
    match RuntimeT::exec(command).await {
        Ok(metadata) => extract_raw_metadata::<RuntimeT>(&metadata).await,
        Err(e) => {
            RuntimeT::log(format!("Failed to generate cargo metadata: {e}")).await;
            MetadataUpdate::NoCargoToml
        }
    }
}

async fn extract_raw_metadata<RuntimeT: Runtime>(raw_metadata: &str) -> MetadataUpdate {
    let Some(metadata) = raw_metadata.lines().find(|line| line.starts_with('{')) else {
        RuntimeT::log("Cargo metadata do not contain valid JSON".to_string()).await;
        return MetadataUpdate::FailedToRetrieve;
    };

    // Parse JSON output into Metadata
    match MetadataCommand::parse(metadata) {
        Ok(metadata) => MetadataUpdate::New(metadata),
        Err(e) => {
            RuntimeT::log(format!("Failed to parse cargo metadata: {e}")).await;
            MetadataUpdate::NoCargoToml
        }
    }
}
