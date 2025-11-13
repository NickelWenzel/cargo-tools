use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use cargo_metadata::{Metadata, MetadataCommand};

use crate::runtime::Runtime;

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Arc<RwLock<Metadata>>),
    NoCargoToml,
    FailedToRetrieve,
}

fn spawn_manifest_handler<RuntimeT: Runtime>(
    metadata_tx: async_broadcast::Sender<Arc<RwLock<Metadata>>>,
    mut manifest_dir_rx: async_broadcast::Receiver<String>,
) -> RuntimeT::ThreadHandle {
    RuntimeT::spawn(async move {
        while let Some(manifest_dir) = manifest_dir_rx.next().await {
            match update_metadata(&manifest_dir).await {
                MetadataUpdate::New(metadata) => {
                    let _ = metadata_tx.broadcast(metadata).await;
                }
                MetadataUpdate::NoCargoToml | MetadataUpdate::FailedToRetrieve => {}
            }
        }
    })
}

async fn update_metadata<RuntimeT: Runtime>(manifest_dir: &str) -> MetadataUpdate {
    // Construct cargo metadata command with manifest path
    let command =
        format!("cargo metadata --format-version 1 --manifest-path {manifest_dir}/Cargo.toml");

    // Execute command via runtime
    let Ok(metadata) = RuntimeT::exec(command)
        .await
        .inspect_err(|e| RuntimeT::log(format!("Failed to generate cargo metadata: {e}")))
    else {
        return MetadataUpdate::NoCargoToml;
    };

    // Convert JsString to Rust String
    let Some(metadata) = metadata.lines().find(|line| line.starts_with('{')) else {
        RuntimeT::log("Cargo metadata do not contain valid JSON".to_string()).await;
        return MetadataUpdate::FailedToRetrieve;
    };

    // Parse JSON output into Metadata
    let Ok(metadata) = MetadataCommand::parse(metadata)
        .inspect_err(|e| RuntimeT::log(format!("Failed to parse cargo metadata: {e}")))
    else {
        return MetadataUpdate::NoCargoToml;
    };

    MetadataUpdate::New(Arc::new(RwLock::new(metadata)))
}

#[derive(Debug, Clone)]
pub struct MakefileTask {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct MakefileTaskCategory;

pub type MakefileTasks = HashMap<MakefileTaskCategory, MakefileTask>;

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(Arc<RwLock<MakefileTasks>>),
    NoMakefile,
    FailedToRetrieve,
}

fn spawn_makefile_handler<RuntimeT: Runtime>(
    makefile_tasks_tx: async_broadcast::Sender<Arc<RwLock<MakefileTasks>>>,
    mut manifest_dir_rx: async_broadcast::Receiver<String>,
) -> RuntimeT::ThreadHandle {
    RuntimeT::spawn(async move {
        while let Some(manifest_dir) = manifest_dir_rx.next().await {
            match update_makefile_tasks(&manifest_dir).await {
                MakefileTasksUpdate::New(makefile_tasks) => {
                    let _ = makefile_tasks_tx.broadcast(makefile_tasks).await;
                }
                MakefileTasksUpdate::NoMakefile | MakefileTasksUpdate::FailedToRetrieve => {}
            }
        }
    })
}

async fn update_makefile_tasks<RuntimeT: Runtime>(manifest_dir: &str) -> MakefileTasksUpdate {
    todo!()
}

pub fn spawn_environment<RuntimeT: Runtime>(
    metadata_tx: async_broadcast::Sender<Arc<RwLock<Metadata>>>,
    makefile_tasks_tx: async_broadcast::Sender<Arc<RwLock<MakefileTasks>>>,
) -> EnvironmentHandles<RuntimeT> {
    let workspace_root_rx = RuntimeT::current_dir_notitifier();
    let manifest_handle = spawn_manifest_handler(metadata_tx, workspace_root_rx.clone());
    let makefile_handle = spawn_makefile_handler(makefile_tasks_tx, workspace_root_rx);

    EnvironmentHandles {
        manifest_handle,
        makefile_handle,
    }
}

pub struct EnvironmentHandles<RuntimeT: Runtime> {
    pub manifest_handle: RuntimeT::ThreadHandle,
    pub makefile_handle: RuntimeT::ThreadHandle,
}
