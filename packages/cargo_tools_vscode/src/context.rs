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

use async_broadcast::{broadcast, Receiver, Sender};
use cargo_tools::app::state::{State, StateUpdate};
use cargo_tools::context::{Configuration, Context};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use wasm_async_trait::wasm_async_trait;
use wasm_bindgen::prelude::*;

use crate::vs_code_api::{self, JsValueExt};

const CHANNEL_CAPACITY: usize = 100;

static PREFIX: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::default()));
static STATE_TX: Lazy<Mutex<Sender<State>>> = Lazy::new(|| {
    let (tx, _) = broadcast(CHANNEL_CAPACITY);
    Mutex::new(tx)
});

static CONFIG: Lazy<Mutex<Option<Configuration>>> = Lazy::new(|| Mutex::new(None));
static CONFIG_TX: Lazy<Mutex<Sender<Configuration>>> = Lazy::new(|| {
    let (tx, _) = broadcast(CHANNEL_CAPACITY);
    Mutex::new(tx)
});

pub struct VsCodeContext;

impl VsCodeContext {
    async fn persist_state(state: State) -> Result<(), String> {
        let full_key = {
            let prefix = PREFIX.lock().unwrap();
            if prefix.is_empty() {
                "state".to_string()
            } else {
                format!("{}.state", *prefix)
            }
        };

        let state = serde_wasm_bindgen::to_value(&state)
            .map_err(|e| format!("Serialization failed: {:?}", e))?;
        vs_code_api::set_state(&full_key, state)
            .await
            .map_err(|e| e.to_error_string())
    }

    fn apply_state_update(state: &mut State, update: StateUpdate) {
        match update {
            StateUpdate::SelectedPackage(v) => state.selected_package = Some(v),
            StateUpdate::SelectedBuildTarget(v) => state.selected_build_target = Some(v),
            StateUpdate::SelectedRunTarget(v) => state.selected_run_target = Some(v),
            StateUpdate::SelectedBenchmarkTarget(v) => state.selected_benchmark_target = Some(v),
            StateUpdate::SelectedPlatformTarget(v) => state.selected_platform_target = Some(v),
            StateUpdate::SelectedFeatures(v) => state.selected_features = Some(v),
            StateUpdate::SelectedProfile(v) => state.selected_profile = Some(v),
            StateUpdate::GroupByWorkspaceMember(v) => state.group_by_workspace_member = Some(v),
            StateUpdate::WorkspaceMemberFilter(v) => state.workspace_member_filter = Some(v),
            StateUpdate::TargetTypeFilter(v) => state.target_type_filter = Some(v),
            StateUpdate::IsTargetTypeFilterActive(v) => {
                state.is_target_type_filter_active = Some(v)
            }
            StateUpdate::ShowFeatures(v) => state.show_features = Some(v),
            StateUpdate::MakefileTaskFilter(v) => state.makefile_task_filter = Some(v),
            StateUpdate::MakefileCategoryFilter(v) => state.makefile_category_filter = Some(v),
            StateUpdate::IsMakefileCategoryFilterActive(v) => {
                state.is_makefile_category_filter_active = Some(v)
            }
            StateUpdate::PinnedMakefileTasks(v) => state.pinned_makefile_tasks = Some(v),
            StateUpdate::Tick => {}
        }
    }

    fn get_configuration() -> Result<Configuration, String> {
        let js_value = vs_code_api::get_configuration();
        serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| format!("Failed to deserialize configuration: {:?}", e))
    }
}

#[wasm_async_trait]
impl Context for VsCodeContext {
    async fn update_prefix(ctx: String) {
        let mut prefix = PREFIX.lock().unwrap();
        *prefix = ctx;
    }

    async fn update_state(update: StateUpdate) {
        let state = {
            let mut state = STATE.lock().unwrap();
            Self::apply_state_update(&mut state, update.clone());
            state.clone()
        };

        if let Err(e) = Self::persist_state(state.clone()).await {
            vs_code_api::log(&format!("Failed persist state in VSCode: {e}"));
        }

        let sender = STATE_TX.lock().unwrap().clone();
        let _ = sender.broadcast(state).await;
    }

    fn state_receiver() -> Receiver<State> {
        let sender = STATE_TX.lock().unwrap();
        sender.new_receiver()
    }
}

#[wasm_bindgen]
pub async fn on_configuration_changed(config_json: String) {
    let new_config: Configuration = match serde_json::from_str(&config_json) {
        Ok(c) => c,
        Err(e) => {
            vs_code_api::log(&format!("Failed to deserialize configuration: {}", e));
            return;
        }
    };

    {
        let mut config = CONFIG.lock().unwrap();
        *config = Some(new_config.clone());
    }

    let sender = CONFIG_TX.lock().unwrap().clone();
    let _ = sender.broadcast(new_config).await;
}
