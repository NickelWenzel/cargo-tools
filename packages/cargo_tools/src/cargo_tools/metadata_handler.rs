use std::sync::{Arc, RwLock};

use cargo_metadata::{Metadata, MetadataCommand};
use futures::{SinkExt, Stream, StreamExt};
use iced_headless::{stream, Subscription, Task};

use crate::{runtime::Runtime, DEFAULT_BUFFER_SIZE};

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Arc<RwLock<Metadata>>),
    NoCargoToml,
    FailedToRetrieve,
}
pub enum MetadataHandlerMessage {
    MetadataUpdate(MetadataUpdate),
}

use MetadataHandlerMessage as Msg;

pub struct MetadataHandler;

impl MetadataHandler {
    pub fn subscription<RuntimeT: Runtime + 'static>(&self) -> Subscription<Msg> {
        Subscription::run(metadata_update::<RuntimeT>).map(Msg::MetadataUpdate)
    }
}

fn metadata_update<RuntimeT: Runtime>() -> impl Stream<Item = MetadataUpdate> {
    stream::channel(DEFAULT_BUFFER_SIZE, async |mut metadata_update_tx| {
        let mut manifest_dir_rx = RuntimeT::current_dir_notitifier();
        while let Some(manifest_dir) = manifest_dir_rx.next().await {
            let metadata_update = update_metadata::<RuntimeT>(manifest_dir).await;
            if metadata_update_tx.send(metadata_update).await.is_err() {
                RuntimeT::log("Failed to notify update metadata update".to_string()).await;
            }
        }
    })
}

pub async fn update_metadata<RuntimeT: Runtime>(manifest_dir: String) -> MetadataUpdate {
    // Construct cargo metadata command with manifest path
    let command = format!(
        "cargo metadata --format-version 1 --manifest-path {manifest_dir}/Cargo.toml --no-deps"
    );

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
        Ok(metadata) => MetadataUpdate::New(Arc::new(RwLock::new(metadata))),
        Err(e) => {
            RuntimeT::log(format!("Failed to parse cargo metadata: {e}")).await;
            MetadataUpdate::NoCargoToml
        }
    }
}
