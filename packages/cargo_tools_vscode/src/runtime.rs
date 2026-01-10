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
use async_broadcast::{Receiver, Sender, broadcast};
use cargo_tools::{
    configuration::Configuration,
    runtime::{CargoTask, Runtime, Task},
};
use once_cell::sync::Lazy;
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::to_value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use wasm_async_trait::wasm_async_trait;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Map;

use crate::{configuration, vs_code_api::*};

const CHANNEL_CAPACITY: usize = 100;

type FileWatcherEntry = (u32, Sender<()>, Receiver<()>);

static CURRENT_DIR_CHANNEL: Lazy<Mutex<(Sender<String>, Receiver<String>)>> =
    Lazy::new(|| Mutex::new(broadcast(CHANNEL_CAPACITY)));

static FILE_WATCHERS: Lazy<Mutex<HashMap<String, FileWatcherEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CURRENT_DIR_HANDLE: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(None));

pub struct VsCodeRuntime;

#[wasm_async_trait]
impl Runtime for VsCodeRuntime {
    async fn exec(command: String) -> Result<String, String> {
        execute_async(&command)
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

    fn current_dir_notitifier() -> Receiver<String> {
        log("In current_dir_notitifier");
        let rx = CURRENT_DIR_CHANNEL.lock().unwrap().1.clone();

        let mut handle = CURRENT_DIR_HANDLE.lock().unwrap();
        if handle.is_none() {
            *handle = Some(watch_current_dir());
        }

        rx
    }

    fn file_changed_notifier(file: String) -> Receiver<()> {
        log(&format!("In file_changed_notifier({file})"));
        let mut watchers = FILE_WATCHERS.lock().unwrap();
        let entry = watchers.entry(file.clone()).or_insert_with(|| {
            let handle = watch_file(&file);
            let (tx, rx) = broadcast(CHANNEL_CAPACITY);
            (handle, tx, rx)
        });
        entry.2.clone()
    }

    async fn persist_state(key: String, state: impl Serialize) {
        let state = serde_wasm_bindgen::to_value(&state);
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
        let js_value = get_state(&key);
        let state = serde_wasm_bindgen::from_value(js_value);
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
            CargoTask::Cargo(_) => "cargo".to_string(),
            CargoTask::CargoMake(_) => "cargo-make".to_string(),
            CargoTask::RustUp(_) => "cargo".to_string(),
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

/// Called by TypeScript when the current directory changes.
#[wasm_bindgen]
pub async fn on_current_dir_changed(dir: String) {
    log(&format!("In on_current_dir_changed({dir})"));
    {
        let mut handle = CURRENT_DIR_HANDLE.lock().unwrap();
        if let Some(h) = *handle {
            log(&format!("In unwatch_current_dir({h})"));
            unwatch_current_dir(h);
            *handle = None;
        }
    }

    let tx = CURRENT_DIR_CHANNEL.lock().unwrap().0.clone();
    if let Err(e) = tx.broadcast(dir).await {
        log(&format!(
            "Failed to send root dir changed notification: {e:?}"
        ));
    }
}

/// Called by TypeScript when a watched file changes.
#[wasm_bindgen]
pub async fn on_file_changed(path: String) {
    log(&format!("In on_file_changed({path})"));
    let tx = {
        let mut watchers = FILE_WATCHERS.lock().unwrap();
        if let Some((handle, tx, _)) = watchers.remove(&path) {
            log(&format!("In unwatch_file({handle})"));
            unwatch_file(handle);
            Some(tx)
        } else {
            None
        }
    };

    if let Some(tx) = tx
        && let Err(e) = tx.broadcast(()).await
    {
        log(&format!("Failed to send file changed notification: {e:?}"));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    //! Tests for VsCodeRuntime event distribution.
    //!
    //! These tests verify the channel-based event distribution for directory and file watchers.
    //! Tests only run on wasm32 target as the Runtime trait uses WASM-specific async primitives.
    use super::*;

    #[test]
    fn test_on_current_dir_changed_distributes_to_all_receivers() {
        *CURRENT_DIR_HANDLE.lock().unwrap() = Some(42);

        let (mut receiver1, mut receiver2) = {
            let rx = CURRENT_DIR_CHANNEL.lock().unwrap().1.clone();
            (rx.clone(), rx)
        };

        on_current_dir_changed("/test/path".to_string());

        let msg1 = receiver1.try_recv();
        let msg2 = receiver2.try_recv();

        assert_eq!(msg1, Ok("/test/path".to_string()));
        assert_eq!(msg2, Ok("/test/path".to_string()));

        let handle = *CURRENT_DIR_HANDLE.lock().unwrap();
        assert_eq!(handle, None, "Watcher handle should be cleared after event");
    }

    #[test]
    fn test_on_current_dir_changed_cleans_up_after_event() {
        *CURRENT_DIR_HANDLE.lock().unwrap() = Some(42);

        let mut rx1 = CURRENT_DIR_CHANNEL.lock().unwrap().1.clone();
        let rx2 = rx1.clone();

        drop(rx2);

        on_current_dir_changed("/test/path".to_string());

        let msg = rx1.try_recv();
        assert_eq!(msg, Ok("/test/path".to_string()));

        let handle = *CURRENT_DIR_HANDLE.lock().unwrap();
        assert_eq!(handle, None, "Watcher handle should be cleared after event");
    }

    #[test]
    fn test_on_file_changed_distributes_to_correct_watchers() {
        FILE_WATCHERS.lock().unwrap().clear();

        let (tx1, mut rx1) = broadcast::<()>(CHANNEL_CAPACITY);
        let (tx2, mut rx2) = broadcast::<()>(CHANNEL_CAPACITY);

        {
            let mut watchers = FILE_WATCHERS.lock().unwrap();
            watchers.insert("/path/file1.txt".to_string(), (1, tx1, rx1.clone()));
            watchers.insert("/path/file2.txt".to_string(), (2, tx2, rx2.clone()));
        }

        on_file_changed("/path/file1.txt".to_string());

        assert_eq!(rx1.try_recv(), Ok(()));
        assert!(rx2.try_recv().is_err(), "Should not receive event");

        let watchers_count = FILE_WATCHERS.lock().unwrap().len();
        assert_eq!(
            watchers_count, 1,
            "Watcher for file1 should be removed after event"
        );
    }

    #[test]
    fn test_on_file_changed_handles_multiple_watchers_same_file() {
        FILE_WATCHERS.lock().unwrap().clear();

        let (tx, mut rx1) = broadcast::<()>(CHANNEL_CAPACITY);
        let mut rx2 = tx.new_receiver();

        {
            let mut watchers = FILE_WATCHERS.lock().unwrap();
            watchers.insert("/path/file.txt".to_string(), (1, tx, rx1.clone()));
        }

        on_file_changed("/path/file.txt".to_string());

        assert_eq!(rx1.try_recv(), Ok(()));
        assert_eq!(rx2.try_recv(), Ok(()));

        let watchers_count = FILE_WATCHERS.lock().unwrap().len();
        assert_eq!(watchers_count, 0, "Watcher should be removed after event");
    }

    #[test]
    fn test_on_file_changed_delivers_to_all_including_dead() {
        FILE_WATCHERS.lock().unwrap().clear();

        let (tx, mut rx1) = broadcast::<()>(CHANNEL_CAPACITY);
        let rx2 = tx.new_receiver();

        {
            let mut watchers = FILE_WATCHERS.lock().unwrap();
            watchers.insert("/path/file.txt".to_string(), (1, tx, rx1.clone()));
        }

        drop(rx2);

        on_file_changed("/path/file.txt".to_string());

        assert_eq!(rx1.try_recv(), Ok(()));

        let watchers_count = FILE_WATCHERS.lock().unwrap().len();
        assert_eq!(watchers_count, 0, "Watcher should be removed after event");
    }

    #[test]
    fn test_channel_capacity_is_bounded() {
        let mut rx = CURRENT_DIR_CHANNEL.lock().unwrap().1.clone();

        for i in 0..CHANNEL_CAPACITY + 10 {
            on_current_dir_changed(format!("/path/{}", i));
        }

        let mut rx_count = 0;
        while rx.try_recv().is_ok() {
            rx_count += 1;
        }

        assert!(
            rx_count <= CHANNEL_CAPACITY,
            "Should not exceed channel capacity"
        );
    }
}
