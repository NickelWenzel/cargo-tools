#[cfg(target_arch = "wasm32")]
pub mod app;
#[cfg(target_arch = "wasm32")]
pub mod configuration;
#[cfg(target_arch = "wasm32")]
pub mod quick_pick;
#[cfg(target_arch = "wasm32")]
pub mod runtime;
#[cfg(target_arch = "wasm32")]
pub mod vs_code_api;

pub mod contributes;
