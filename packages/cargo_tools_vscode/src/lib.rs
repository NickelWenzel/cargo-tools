#[cfg(target_arch = "wasm32")]
pub mod environment;
#[cfg(target_arch = "wasm32")]
pub mod extension;
#[cfg(target_arch = "wasm32")]
pub mod icon;

/// Helpers to setup VS Code quick pick menus
#[cfg(target_arch = "wasm32")]
pub mod quick_pick;
#[cfg(target_arch = "wasm32")]
pub mod runtime;
#[cfg(target_arch = "wasm32")]
pub mod vs_code_api;

pub mod commands;
