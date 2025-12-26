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
use async_broadcast::{broadcast, Receiver, Sender};
use cargo_tools::{
    contributes::Configuration,
    runtime::{CargoTask, Runtime},
};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use wasm_async_trait::wasm_async_trait;
use wasm_bindgen::prelude::*;

use crate::vs_code_api::{self, JsValueExt};

const CHANNEL_CAPACITY: usize = 100;

type FileWatcherEntry = (u32, Sender<()>);

static CURRENT_DIR_TX: Lazy<Mutex<Sender<String>>> = Lazy::new(|| {
    let (tx, _) = broadcast(CHANNEL_CAPACITY);
    Mutex::new(tx)
});

static FILE_WATCHERS: Lazy<Mutex<HashMap<String, FileWatcherEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CURRENT_DIR_HANDLE: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(None));

pub struct VsCodeRuntime;

#[wasm_async_trait]
impl Runtime for VsCodeRuntime {
    async fn exec(command: String) -> Result<String, String> {
        vs_code_api::execute_async(&command)
            .await
            .map(|js_str| js_str.as_string().expect("JsString conversion failed"))
            .map_err(|e| e.to_error_string())
    }
    async fn exec_task(_task: CargoTask) {
        todo!()
    }

    async fn log(msg: String) {
        vs_code_api::log(&msg);
    }

    fn current_dir_notitifier() -> Receiver<String> {
        let sender = CURRENT_DIR_TX.lock().unwrap();
        let receiver = sender.new_receiver();

        let mut handle = CURRENT_DIR_HANDLE.lock().unwrap();
        if handle.is_none() {
            *handle = Some(vs_code_api::watch_current_dir());
        }

        receiver
    }

    fn file_changed_notifier(file: String) -> Receiver<()> {
        let mut watchers = FILE_WATCHERS.lock().unwrap();
        let entry = watchers.entry(file.clone()).or_insert_with(|| {
            let handle = vs_code_api::watch_file(&file);
            let (sender, _) = broadcast(CHANNEL_CAPACITY);
            (handle, sender)
        });
        entry.1.new_receiver()
    }

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

/// Called by TypeScript when the current directory changes.
#[wasm_bindgen]
pub async fn on_current_dir_changed(dir: String) {
    {
        let mut handle = CURRENT_DIR_HANDLE.lock().unwrap();
        if let Some(h) = *handle {
            vs_code_api::unwatch_current_dir(h);
            *handle = None;
        }
    }

    let sender = CURRENT_DIR_TX.lock().unwrap().clone();
    let _ = sender.broadcast(dir).await;
}

/// Called by TypeScript when a watched file changes.
#[wasm_bindgen]
pub fn on_file_changed(path: String) {
    let mut watchers = FILE_WATCHERS.lock().unwrap();
    if let Some((handle, sender)) = watchers.remove(&path) {
        vs_code_api::unwatch_file(handle);
        let _ = sender.try_broadcast(());
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
            let sender = CURRENT_DIR_TX.lock().unwrap();
            (sender.new_receiver(), sender.new_receiver())
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

        let sender = CURRENT_DIR_TX.lock().unwrap();
        let mut receiver1 = sender.new_receiver();
        let receiver2 = sender.new_receiver();
        drop(sender);

        drop(receiver2);

        on_current_dir_changed("/test/path".to_string());

        let msg = receiver1.try_recv();
        assert_eq!(msg, Ok("/test/path".to_string()));

        let handle = *CURRENT_DIR_HANDLE.lock().unwrap();
        assert_eq!(handle, None, "Watcher handle should be cleared after event");
    }

    #[test]
    fn test_on_file_changed_distributes_to_correct_watchers() {
        FILE_WATCHERS.lock().unwrap().clear();

        let (sender1, _) = broadcast::<()>(CHANNEL_CAPACITY);
        let mut receiver1 = sender1.new_receiver();
        let (sender2, _) = broadcast::<()>(CHANNEL_CAPACITY);
        let mut receiver2 = sender2.new_receiver();

        {
            let mut watchers = FILE_WATCHERS.lock().unwrap();
            watchers.insert("/path/file1.txt".to_string(), (1, sender1));
            watchers.insert("/path/file2.txt".to_string(), (2, sender2));
        }

        on_file_changed("/path/file1.txt".to_string());

        assert_eq!(receiver1.try_recv(), Ok(()));
        assert!(receiver2.try_recv().is_err(), "Should not receive event");

        let watchers_count = FILE_WATCHERS.lock().unwrap().len();
        assert_eq!(
            watchers_count, 1,
            "Watcher for file1 should be removed after event"
        );
    }

    #[test]
    fn test_on_file_changed_handles_multiple_watchers_same_file() {
        FILE_WATCHERS.lock().unwrap().clear();

        let (sender, _) = broadcast::<()>(CHANNEL_CAPACITY);
        let mut receiver1 = sender.new_receiver();
        let mut receiver2 = sender.new_receiver();

        {
            let mut watchers = FILE_WATCHERS.lock().unwrap();
            watchers.insert("/path/file.txt".to_string(), (1, sender));
        }

        on_file_changed("/path/file.txt".to_string());

        assert_eq!(receiver1.try_recv(), Ok(()));
        assert_eq!(receiver2.try_recv(), Ok(()));

        let watchers_count = FILE_WATCHERS.lock().unwrap().len();
        assert_eq!(watchers_count, 0, "Watcher should be removed after event");
    }

    #[test]
    fn test_on_file_changed_delivers_to_all_including_dead() {
        FILE_WATCHERS.lock().unwrap().clear();

        let (sender, _) = broadcast::<()>(CHANNEL_CAPACITY);
        let mut receiver1 = sender.new_receiver();
        let receiver2 = sender.new_receiver();

        {
            let mut watchers = FILE_WATCHERS.lock().unwrap();
            watchers.insert("/path/file.txt".to_string(), (1, sender));
        }

        drop(receiver2);

        on_file_changed("/path/file.txt".to_string());

        assert_eq!(receiver1.try_recv(), Ok(()));

        let watchers_count = FILE_WATCHERS.lock().unwrap().len();
        assert_eq!(watchers_count, 0, "Watcher should be removed after event");
    }

    #[test]
    fn test_channel_capacity_is_bounded() {
        let sender = CURRENT_DIR_TX.lock().unwrap();
        let mut receiver = sender.new_receiver();
        drop(sender);

        for i in 0..CHANNEL_CAPACITY + 10 {
            on_current_dir_changed(format!("/path/{}", i));
        }

        let mut received_count = 0;
        while receiver.try_recv().is_ok() {
            received_count += 1;
        }

        assert!(
            received_count <= CHANNEL_CAPACITY,
            "Should not exceed channel capacity"
        );
    }
}
