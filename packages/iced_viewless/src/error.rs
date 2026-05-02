//! Error types for the iced_viewless runtime.

use thiserror::Error;

/// Errors that can occur when running a viewless application.
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to create the executor for running async tasks.
    #[error("Failed to create executor")]
    ExecutorCreationFailed(#[source] futures::io::Error),

    /// The runtime encountered an error during execution.
    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
