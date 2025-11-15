pub mod cargo_tools;
pub mod configuration_handler;
pub mod state;

pub mod application;
pub mod environment;
pub mod runtime;

/// Default buffer size for broadcast channels used throughout the crate.
/// This ensures consistent sizing for state, settings, metadata, and makefile task broadcasts.
pub const DEFAULT_BUFFER_SIZE: usize = 100;
