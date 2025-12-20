use std::{iter, sync::Arc};

use cargo_metadata::{Metadata, MetadataCommand};
use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::runtime::Runtime;

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Arc<Metadata>),
    NoCargoToml,
    FailedToRetrieve,
}
pub enum CargoSettingsMessage {
    RootDirChanged(String),
    ManifestChanged,
    MetadataUpdate(MetadataUpdate),
}

use CargoSettingsMessage as Msg;

pub struct CargoSettings {
    root_manifest: String,
    member_manifests: Vec<String>,
}

impl CargoSettings {
    pub fn update<RuntimeT: Runtime>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirChanged(root_dir) => {
                self.root_manifest = format!("{root_dir}/Cargo.toml");
                Task::future(update_metadata::<RuntimeT>(self.root_manifest.clone()))
                    .map(Msg::MetadataUpdate)
            }
            Msg::ManifestChanged => {
                Task::future(update_metadata::<RuntimeT>(self.root_manifest.clone()))
                    .map(Msg::MetadataUpdate)
            }
            Msg::MetadataUpdate(update) => {
                match update {
                    MetadataUpdate::New(metadata) => Task::future(async move {
                        let file_changed = iter::once(metadata.workspace_root.to_string())
                            .chain(to_manifest_files(&metadata))
                            .map(RuntimeT::file_changed_notifier);
                        futures::stream::select_all(file_changed).next().await;
                    })
                    .map(|()| Msg::ManifestChanged),
                    MetadataUpdate::NoCargoToml => {
                        // Reset
                        self.member_manifests = Vec::new();
                        Task::none()
                    }
                    // Leave files to watch unchanged
                    MetadataUpdate::FailedToRetrieve => Task::none(),
                }
            }
        }
    }

    pub fn subscription<RuntimeT: Runtime>(&self) -> Subscription<Msg> {
        Subscription::run(RuntimeT::current_dir_notitifier).map(Msg::RootDirChanged)
    }
}

fn to_manifest_files(metadata: &Metadata) -> impl Iterator<Item = String> + use<'_> {
    metadata
        .workspace_packages()
        .into_iter()
        .map(|p| p.manifest_path.to_string())
}

pub async fn update_metadata<RuntimeT: Runtime>(manifest_file: String) -> MetadataUpdate {
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
        Ok(metadata) => MetadataUpdate::New(Arc::new(metadata)),
        Err(e) => {
            RuntimeT::log(format!("Failed to parse cargo metadata: {e}")).await;
            MetadataUpdate::NoCargoToml
        }
    }
}
