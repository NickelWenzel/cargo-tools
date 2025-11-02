use cargo_metadata::{Error, Metadata, MetadataCommand};
use wasm_bindgen::prelude::*;

use crate::{
    state_manager::StateManager,
    vs_code_api::{echo_task, execute_async, log, set_cargo_context, set_makefile_context},
};

#[wasm_bindgen]
pub struct CargoTools {
    metadata: Metadata,
    state_manager: StateManager,
}

#[derive(Debug)]
pub enum CargoToolsError {
    AsyncExecution(JsValue),
    MetadataConversion(String),
    NoJsonOutput,
    MetadataParsing(Error),
}

impl std::fmt::Display for CargoToolsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoToolsError::AsyncExecution(js_val) => {
                write!(f, "Async execution failed: {:?}", js_val)
            }
            CargoToolsError::MetadataConversion(msg) => {
                write!(f, "Metadata conversion failed: {}", msg)
            }
            CargoToolsError::NoJsonOutput => {
                write!(f, "No JSON output found in cargo metadata command")
            }
            CargoToolsError::MetadataParsing(err) => {
                write!(f, "Failed to parse cargo metadata: {}", err)
            }
        }
    }
}

impl std::error::Error for CargoToolsError {}

impl From<CargoToolsError> for JsValue {
    fn from(err: CargoToolsError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

impl CargoTools {
    pub async fn create(
        workspace_root: &str,
        state_manager: StateManager,
    ) -> Result<Self, JsValue> {
        match Self::create_impl(workspace_root, state_manager).await {
            Ok(instance) => {
                set_cargo_context(true);
                Ok(instance)
            }
            Err(e) => {
                log(&format!("Error creating CargoTools instance: {}", e));
                set_cargo_context(false);
                set_makefile_context(false);
                Err(e.into())
            }
        }
    }

    pub async fn test(&self) {
        echo_task("Test echo task from CargoTools").await;
    }

    async fn create_impl(
        workspace_root: &str,
        state_manager: StateManager,
    ) -> Result<Self, CargoToolsError> {
        log("Creating new CargoTools instance");

        let metadata = execute_async(
            "cargo metadata --no-deps --format-version 1",
            workspace_root,
        )
        .await
        .map_err(CargoToolsError::AsyncExecution)?;

        let metadata = metadata
            .as_string()
            .ok_or(CargoToolsError::MetadataConversion(
                "Failed to convert metadata JsString to Rust String".to_string(),
            ))?;

        let metadata = metadata
            .lines()
            .find(|line| line.starts_with('{'))
            .ok_or(CargoToolsError::NoJsonOutput)?;

        MetadataCommand::parse(metadata)
            .map(|metadata| CargoTools {
                metadata,
                state_manager,
            })
            .map_err(CargoToolsError::MetadataParsing)
    }
}
