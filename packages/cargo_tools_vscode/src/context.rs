//! VS Code Context implementation for state and configuration management.
//!
//! This module provides the concrete implementation of the `Context` trait for VS Code,
//! bridging Rust/WASM code to VS Code's workspace state and configuration APIs.
//!
//! # Architecture
//!
//! - **State Management**: Uses VS Code workspace state (Memento API) for persistent storage
//!   - State is keyed with dynamic prefixes controlled by `update_state_context()`
//!   - Each state field is stored separately for granular updates
//!   - Channel-based broadcasting to multiple subscribers
//! - **Configuration Management**: Uses VS Code configuration API for extension settings
//!   - Configuration read from `cargoTools` section
//!   - Updates modify specific properties in the configuration object
//!   - Channel-based broadcasting for configuration changes
//!
//! # Key Prefix Pattern
//!
//! The context methods allow changing the key prefix dynamically:
//! - `update_state_context("workspace1")` sets prefix to "workspace1."
//! - State key "selected_package" becomes "workspace1.selected_package"
//! - Allows isolation of state per workspace/project
//!
//! # TypeScript Integration
//!
//! Requires TypeScript module `state.ts` with:
//! - `get_state_value(key)` - Read from workspace state
//! - `set_state_value(key, value)` - Write to workspace state
//! - `get_configuration_json()` - Read full configuration
//! - `set_configuration_value(key, value)` - Update configuration property

use std::fmt::Debug;

use cargo_tools::context::Context;
use cargo_tools::contributes::Configuration;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_async_trait::wasm_async_trait;

use crate::vs_code_api::{self, JsValueExt};

pub struct VsCodeContext;

#[wasm_async_trait]
impl Context for VsCodeContext {
    async fn persist_state(key: String, state: impl Serialize) {
        let state = serde_wasm_bindgen::to_value(&state);
        let Ok(state) = state else {
            let e = state.unwrap_err();
            vs_code_api::log(&format!("Failed to serialize state: {e}"));
            return;
        };

        if let Err(e) = vs_code_api::set_state(&key, state).await {
            let e = e.to_error_string();
            vs_code_api::log(&format!("Failed to set state: {e}"));
        }
    }

    fn get_state<T: DeserializeOwned + Debug>(key: String) -> Option<T> {
        let js_value = vs_code_api::get_state(&key);
        let state = serde_wasm_bindgen::from_value(js_value);
        let Ok(state) = state else {
            let e = state.unwrap_err();
            vs_code_api::log(&format!("Failed to deserialize state: {e}"));
            return None;
        };
        Some(state)
    }

    fn get_configuration() -> Option<Configuration> {
        let js_value = vs_code_api::get_configuration();
        let conf = serde_wasm_bindgen::from_value(js_value);
        let Ok(conf) = conf else {
            let e = conf.unwrap_err();
            vs_code_api::log(&format!("Failed to deserialize configuration: {e}"));
            return None;
        };
        Some(conf)
    }
}
