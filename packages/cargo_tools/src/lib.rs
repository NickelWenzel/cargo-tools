pub mod app;

pub mod context;
pub mod contributes;
pub mod runtime;

/// Default buffer size for broadcast channels used throughout the crate.
/// This ensures consistent sizing for state, settings, metadata, and makefile task broadcasts.
pub const DEFAULT_BUFFER_SIZE: usize = 100;
