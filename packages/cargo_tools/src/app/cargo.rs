use cargo_metadata::{Metadata, MetadataCommand};
use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{app::selection::Selection, context::Context, runtime::Runtime};

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Metadata),
    NoCargoToml,
    FailedToRetrieve,
}
pub enum CargoMessage {
    RootDirUpdate(String),
    SelectionUpdate(Selection),
    ManifestUpdate,
    MetadataUpdate(MetadataUpdate),
}

use CargoMessage as Msg;

pub struct Cargo {
    root_manifest: String,
    workspace_manifests: Vec<String>,
    metadata: MetadataUpdate,
    selection: Selection,
}

impl Cargo {
    pub fn update<RT: Runtime, CTX: Context>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT, CTX>(root_dir),
            Msg::ManifestUpdate => Task::future(parse_metadata::<RT>(self.root_manifest.clone()))
                .map(Msg::MetadataUpdate),
            Msg::MetadataUpdate(update) => self.update_metadata::<RT>(update),
            Msg::SelectionUpdate(selection) => {
                self.selection = selection;
                Task::none()
            }
        }
    }

    fn update_root_dir<RT: Runtime, CTX: Context>(&mut self, root_dir: String) -> Task<Msg> {
        self.root_manifest = format!("{root_dir}/Cargo.toml");
        let selection = {
            if let Some(s) = CTX::get_state(format!("{root_dir}.cargo_selection")) {
                Task::done(Msg::SelectionUpdate(s))
            } else {
                Task::none()
            }
        };

        let metadata =
            Task::future(parse_metadata::<RT>(self.root_manifest.clone())).map(Msg::MetadataUpdate);

        Task::batch([metadata, selection])
    }

    fn update_metadata<RT: Runtime>(&mut self, metadata_update: MetadataUpdate) -> Task<Msg> {
        match &metadata_update {
            MetadataUpdate::New(metadata) => {
                self.workspace_manifests = workspace_manifests(metadata);
            }
            MetadataUpdate::NoCargoToml => {
                self.workspace_manifests = Vec::new();
            }
            MetadataUpdate::FailedToRetrieve => {}
        }

        self.metadata = metadata_update;

        let manifests_changed = self.manifests();
        Task::future(async move {
            let notifiers = manifests_changed.into_iter().map(RT::file_changed_notifier);
            futures::stream::select_all(notifiers).next().await;
        })
        .map(|()| Msg::ManifestUpdate)
    }

    pub fn subscription<RuntimeT: Runtime>(&self) -> Subscription<Msg> {
        Subscription::run(RuntimeT::current_dir_notitifier).map(Msg::RootDirUpdate)
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
