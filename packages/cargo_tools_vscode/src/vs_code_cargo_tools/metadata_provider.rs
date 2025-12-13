use cargo_metadata::{Metadata, MetadataCommand};
use thiserror::Error;
use wasm_async_trait::wasm_async_trait;

use crate::vs_code_api::{execute_async, JsValueExt};

/// VS Code-specific implementation of MetaDataProvider.
///
/// This struct provides access to cargo metadata through the VS Code extension API.
/// It executes `cargo metadata` commands via the VS Code extension host and parses
/// the results into structured metadata.
///
/// # Examples
///
/// ```no_run
/// use cargo_tools::cargo_tools::MetaDataProvider;
/// use cargo_tools_vscode::vs_code_cargo_tools::metadata_provider::VSCodeMetaDataProvider;
///
/// async fn get_metadata() -> Result<(), Box<dyn std::error::Error>> {
///     let provider = VSCodeMetaDataProvider;
///     let metadata = provider.request("/path/to/workspace").await?;
///     println!("Workspace root: {}", metadata.workspace_root);
///     Ok(())
/// }
/// ```
pub struct VSCodeMetaDataProvider;

/// Error type for VSCodeMetaDataProvider operations.
///
/// This enum represents all possible errors that can occur when requesting
/// cargo metadata through the VS Code extension API.
#[derive(Error, Debug)]
pub enum MetaDataProviderError {
    /// Error executing metadata command through VS Code API.
    ///
    /// This can occur if:
    /// - The cargo executable is not found
    /// - The manifest path is invalid
    /// - The command times out
    /// - The VS Code API call fails
    #[error("Failed to execute cargo metadata command: {0}")]
    CommandExecution(String),

    /// No JSON found in command output.
    ///
    /// This occurs when the cargo metadata command produces output
    /// but no line starting with '{' (indicating JSON) was found.
    /// This typically happens when cargo produces only error messages
    /// or warnings without valid metadata output.
    #[error("No JSON metadata found in cargo output")]
    NoJson,

    /// Error parsing metadata JSON output.
    ///
    /// This occurs when cargo metadata produces output that cannot be parsed
    /// into the expected `Metadata` structure, typically indicating a version
    /// mismatch or corrupted output.
    #[error("Failed to parse cargo metadata")]
    ParseError(#[source] cargo_metadata::Error),
}

#[wasm_async_trait]
impl cargo_tools::cargo_tools::CargoTomlHandler for VSCodeMetaDataProvider {
    type Error = MetaDataProviderError;

    async fn request(&self, path: &str) -> Result<Metadata, Self::Error> {
        // Construct cargo metadata command with manifest path
        let command = format!(
            "cargo metadata --format-version 1 --manifest-path {}/Cargo.toml",
            path
        );

        // Execute command via VS Code API
        let js_string = execute_async(&command)
            .await
            .map_err(|e| MetaDataProviderError::CommandExecution(e.as_error_string()))?;

        // Convert JsString to Rust String
        let output = String::from(js_string);
        let json_line = output
            .lines()
            .find(|line| line.starts_with('{'))
            .ok_or(MetaDataProviderError::NoJson)?;

        // Parse JSON output into Metadata
        MetadataCommand::parse(json_line).map_err(MetaDataProviderError::ParseError)
    }
}
