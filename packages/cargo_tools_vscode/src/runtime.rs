//! VS Code Runtime implementation for executing commands and watching file system events.
//!
//! This module provides the concrete implementation of the `Runtime` trait for VS Code,
//! bridging Rust/WASM code to VS Code's TypeScript APIs for command execution and
//! file system watching.
//!
//! # Architecture
//!
//! - **Command Execution**: Delegates to `vs_code_api::execute_async` which calls TypeScript
//! - **Logging**: Delegates to `vs_code_api::log` which uses VS Code's console API
//! - **File Watching**: Uses bounded channels (capacity 100) with multi-subscriber support
//!   - Directory changes are broadcast to all `current_dir_notitifier()` subscribers
//!   - File changes are routed to subscribers of specific file paths
//!   - **One-time events**: Watchers are automatically disposed after the first event fires
//!   - Dead receivers are automatically cleaned up on send failures
//!
//! # TypeScript Integration
//!
//! This module requires corresponding TypeScript implementations in `runtime.ts`:
//! - `watch_current_dir()` - Creates VS Code workspace folder watcher, returns handle
//! - `unwatch_current_dir(handle)` - Disposes watcher by handle
//! - `watch_file(path)` - Creates VS Code file system watcher for specific path, returns handle
//! - `unwatch_file(handle)` - Disposes file watcher by handle
//!
//! TypeScript must call `on_current_dir_changed(dir)` and `on_file_changed(path)`
//! when events occur to propagate changes to Rust subscribers. After calling these
//! functions, Rust will automatically call the unwatch functions to dispose of watchers.
//!
//! # Testing
//!
//! Unit tests are provided but cannot be executed directly on wasm32 target.
//! They serve as documentation and can be validated through integration tests
//! or manual testing in the VS Code extension.
use cargo_tools::{
    configuration::Configuration,
    runtime::{CargoTask, Runtime, Task},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;
use wasm_async_trait::wasm_async_trait;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Map;

use crate::{configuration, vs_code_api::*};

pub const CHANNEL_CAPACITY: usize = 100;

pub struct VsCodeRuntime;

#[wasm_async_trait]
impl Runtime for VsCodeRuntime {
    async fn exec(command: String, args: Vec<String>) -> Result<String, String> {
        execute_async(&command, args)
            .await
            .map(|js_str| js_str.as_string().expect("JsString conversion failed"))
            .map_err(|e| e.to_error_string())
    }

    async fn exec_task(task: CargoTask) {
        execute_task(VsCodeTask(task)).await;
    }

    fn log(msg: String) {
        log(&msg);
    }

    async fn read_file(file_path: String) -> Result<String, String> {
        read_file(&file_path)
            .await
            .map(|js_str| js_str.as_string().expect("JsString conversion failed"))
            .map_err(|e| e.to_error_string())
    }

    async fn persist_state(key: String, state: impl Serialize) {
        let state = serde_json::to_string(&state);
        let Ok(state) = state else {
            let e = state.unwrap_err();
            log(&format!("Failed to serialize state: {e}"));
            return;
        };

        if let Err(e) = set_state(&key, state).await {
            let e = e.to_error_string();
            log(&format!("Failed to set state: {e}"));
        }
    }

    fn get_state<T: DeserializeOwned + Debug>(key: String) -> Option<T> {
        let state = get_state(&key);
        let Ok(state) = state else {
            log(&format!(
                "Failed to get state: {}",
                state.unwrap_err().to_error_string()
            ));
            return None;
        };
        let state = serde_json::from_str(&state);
        let Ok(state) = state else {
            let e = state.unwrap_err();
            log(&format!("Failed to deserialize state: {e}"));
            return None;
        };
        Some(state)
    }

    fn get_configuration() -> impl Configuration {
        configuration::Configuration
    }
}

/// Task type which is exported in typescript code
#[wasm_bindgen]
pub struct VsCodeTask(CargoTask);

#[wasm_bindgen]
impl VsCodeTask {
    #[wasm_bindgen]
    pub fn task_type(&self) -> String {
        match self.0 {
            CargoTask::Cargo(_) => "cargo-tools-cargo".to_string(),
            CargoTask::CargoMake(_) => "cargo-tools-cargo-make".to_string(),
            CargoTask::RustUp(_) => "cargo-tools-cargo".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn cmd(&self) -> String {
        self.task().cmd.clone()
    }

    #[wasm_bindgen]
    pub fn args(&self) -> Vec<String> {
        self.task().args.clone()
    }

    #[wasm_bindgen]
    pub fn env(&self) -> Map {
        to_value(&self.task().env)
            .map(Map::from)
            .unwrap_or_default()
    }

    fn task(&self) -> &Task {
        match &self.0 {
            CargoTask::Cargo(task) => task,
            CargoTask::CargoMake(task) => task,
            CargoTask::RustUp(task) => task,
        }
    }
}
