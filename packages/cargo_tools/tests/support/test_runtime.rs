//! Test implementation of the Runtime trait for integration testing.
//!
//! This module provides a concrete Runtime implementation that bridges to the
//! tracing logging framework, enabling log verification in tests via tracing-test.

use std::{future::Future, pin::Pin};

use async_broadcast::Receiver;
use cargo_tools::runtime::Runtime;
use cargo_tools_macros::wasm_async_trait;

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
    type ThreadHandle = Pin<Box<dyn Future<Output = ()> + Send>>;

    fn spawn<Result, F>(_f: F) -> Self::ThreadHandle
    where
        Self::ThreadHandle: Future<Output = Result>,
    {
        // For testing, we don't actually spawn threads
        Box::pin(async {})
    }

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

    async fn current_dir_notitifier() -> Receiver<String> {
        // Return a mock receiver for testing
        let (_, rx) = async_broadcast::broadcast(1);
        rx
    }
}
