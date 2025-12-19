//! Test implementation of the Runtime trait for integration testing.
//!
//! This module provides a concrete Runtime implementation that bridges to the
//! tracing logging framework, enabling log verification in tests via tracing-test.

use cargo_tools::{
    cargo_tools::state::{State, StateUpdate},
    runtime::{Runtime, Settings, SettingsUpdate},
};
use futures::channel::mpsc::Receiver;
use wasm_async_trait::wasm_async_trait;

/// Test runtime implementation for integration testing.
///
/// This zero-sized type implements the Runtime trait using native system calls
/// via cmd_lib for command execution and tracing for logging. It enables testing
/// of platform-agnostic code without requiring WASM infrastructure.
///
/// # Example
///
/// ```no_run
/// use test_runtime::TestRuntime;
/// use cargo_tools::runtime::Runtime;
///
/// async fn example() {
///     let result = TestRuntime::exec("echo hello".to_string()).await;
///     assert!(result.is_ok());
/// }
/// ```
#[derive(Debug)]
pub struct TestRuntime;

#[wasm_async_trait]
impl Runtime for TestRuntime {
    async fn exec(command: String) -> Result<String, String> {
        // Execute command using sh -c wrapper to allow dynamic command strings
        // This is necessary because cmd_lib's run_fun! macro requires literal syntax
        let result = cmd_lib::run_fun!(sh -c $command).map_err(|e| e.to_string())?;
        Ok(result)
    }

    async fn log(msg: String) {
        // Bridge Runtime::log to tracing for test verification
        tracing::info!("{}", msg);
    }

    fn current_dir_notitifier() -> Receiver<String> {
        // Return a mock receiver for testing
        let (_, rx) = futures::channel::mpsc::channel(1);
        rx
    }

    fn file_changed_notifier(_file: String) -> Receiver<()> {
        todo!()
    }

    async fn update_state_context(_ctx: String) -> State {
        // Return default state for testing
        State::default()
    }

    async fn update_state(_update: StateUpdate) -> State {
        // Return default state for testing
        State::default()
    }

    async fn update_settings_context(_ctx: String) -> Settings {
        // Return default settings for testing
        Settings
    }

    async fn update_settings(_update: SettingsUpdate) -> Settings {
        // Return default settings for testing
        Settings
    }
}
