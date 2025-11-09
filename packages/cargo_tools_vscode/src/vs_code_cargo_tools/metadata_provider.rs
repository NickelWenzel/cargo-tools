use cargo_metadata::Metadata;
use thiserror::Error;

/// VS Code-specific implementation of MetaDataProvider.
///
/// This struct provides access to cargo metadata through the VS Code extension API.
pub struct VSCodeMetaDataProvider;

impl VSCodeMetaDataProvider {
    /// Create a new VSCodeMetaDataProvider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VSCodeMetaDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for VSCodeMetaDataProvider operations.
#[derive(Error, Debug)]
pub enum MetaDataProviderError {
    /// Error executing metadata command
    #[error("Failed to execute cargo metadata command: {0}")]
    CommandExecution(String),

    /// Error parsing metadata
    #[error("Failed to parse cargo metadata")]
    ParseError(#[source] cargo_metadata::Error),
}

impl cargo_tools::cargo_tools::MetaDataProvider for VSCodeMetaDataProvider {
    type Error = MetaDataProviderError;

    fn request(&self, _path: &str) -> Result<Metadata, Self::Error> {
        todo!("Implement cargo metadata request through VS Code API")
    }
}
