#[cfg(target_arch = "wasm32")]
pub mod environment;
#[cfg(target_arch = "wasm32")]
pub mod extension;
#[cfg(target_arch = "wasm32")]
pub mod icon;
#[cfg(target_arch = "wasm32")]
pub mod logger;

/// Helpers to setup VS Code quick pick menus
#[cfg(target_arch = "wasm32")]
pub mod quick_pick;

/// Interactions with the host's VS Code and Node runtime
/// Read files, execute processes and VS Code tasks
#[cfg(target_arch = "wasm32")]
pub mod runtime;

pub mod commands;
