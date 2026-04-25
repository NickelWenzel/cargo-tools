//! Test implementation of the Runtime trait for integration testing.
//!
//! This module provides a concrete Runtime implementation that bridges to the
//! tracing logging framework, enabling log verification in tests via tracing-test.
use async_broadcast::Receiver;
use cargo_tools::{
    environment::{self, Environment},
    runtime::{CargoTask, Runtime, Task},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, fmt::Debug};
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

    async fn exec_task(task: CargoTask) {
        // Execute command using sh -c wrapper to allow dynamic command strings
        // This is necessary because cmd_lib's run_fun! macro requires literal syntax
        let Task { cmd, args, env } = match task {
            CargoTask::Cargo(task) => task,
            CargoTask::CargoMake(task) => task,
            CargoTask::RustUp(task) => task,
        };
        let env = env
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ");
        let args = args.join(" ");
        let _ = cmd_lib::run_cmd!(sh -c "${env} ${cmd} ${args}").map_err(|e| e.to_string());
    }

    fn log(msg: String) {
        // Bridge Runtime::log to tracing for test verification
        tracing::info!("{}", msg);
    }

    async fn persist_state(_key: String, _state: impl Serialize + Send) {}

    fn get_state<T: DeserializeOwned + Debug>(_key: String) -> Option<T> {
        None
    }

    async fn read_file(file_path: String) -> Result<String, String> {
        Err("Not implemented".to_string())
    }
}

pub struct TestConfig;

impl Environment for TestConfig {
    fn get_env(&self, _: cargo_tools::environment::Context) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_extra_args(&self, _: cargo_tools::environment::Context) -> Vec<String> {
        Vec::new()
    }

    fn get_cargo_command(&self, _: cargo_tools::environment::Context) -> String {
        "cargo".to_string()
    }
}
