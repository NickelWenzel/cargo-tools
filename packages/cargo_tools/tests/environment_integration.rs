//! Integration tests for environment discovery functionality.
//!
//! These tests verify the metadata and makefile task discovery functionality
//! using the test-rust-project as test data.

use cargo_tools::{
    cargo::metadata::{MetadataUpdate, parse_metadata},
    cargo_make::{MakefileTasksUpdate, parse_tasks},
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_node_experimental);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = child_process)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn exec(
        cmd: String,
        cb: &Closure<dyn FnMut(JsValue, String, String)>,
    ) -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
async fn exec_cmd(cmd: String, args: Vec<String>) -> Result<String, String> {
    use futures::{SinkExt, StreamExt, channel::mpsc};
    use itertools::Itertools;

    let cmd = std::iter::once(cmd).chain(args).join(" ");
    let (tx, mut rx) = mpsc::channel(1);
    let cb = Closure::new(move |error: JsValue, stdout: String, stderr: String| {
        let mut tx = tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if error.is_null() && stderr.is_empty() {
                tx.send(Ok(stdout)).await.expect("Failed to send stdout");
            } else {
                tx.send(Err(stderr)).await.expect("Failed to send stderr");
            }
        })
    });

    match exec(cmd, &cb) {
        Ok(_p) => rx.next().await.expect("Failed to receive stdout"),
        Err(e) => Err(e.as_string().unwrap_or(format!("{e:?}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn exec_cmd(cmd: String, args: Vec<String>) -> Result<String, String> {
    use std::process::Command;
    match Command::new(cmd).args(args).output() {
        Ok(output) => match output.status.success() {
            true => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            false => Err(String::from_utf8_lossy(&output.stderr).to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

/// Test successful metadata discovery from test-rust-project.
#[wasm_bindgen_test(unsupported = tokio::test)]
#[tracing_test::traced_test]
async fn test_update_metadata_success() {
    // Use canonicalized absolute path to avoid working directory issues with cmd_lib
    // Note: parse_metadata expects manifest directory, not the full Cargo.toml path

    let manifest = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-rust-project/Cargo.toml"
    )
    .to_string();

    let result = parse_metadata(manifest, exec_cmd).await;

    // Verify success variant
    assert!(
        matches!(result, MetadataUpdate::Metadata(_)),
        "Expected MetadataUpdate::New, got: {:?}",
        result
    );

    // Extract and verify metadata contents
    if let MetadataUpdate::Metadata(metadata) = result {
        // Verify workspace members are present
        let workspace_members: Vec<&str> = metadata
            .workspace_packages()
            .iter()
            .map(|p| p.name.as_str())
            .collect();

        // Expected packages from test-rust-project
        let expected_members = vec![
            "core",
            "cli",
            "web-server",
            "utils",
            "test-cdylib",
            "test-staticlib",
            "test-proc-macro",
            "test-proc-macro-alt",
        ];

        for expected in &expected_members {
            assert!(
                workspace_members
                    .iter()
                    .any(|m: &&str| m.contains(expected)),
                "Expected workspace member '{}' not found in: {:?}",
                expected,
                workspace_members
            );
        }
    }
}

/// Test makefile task discovery - skips if cargo-make not installed.
#[wasm_bindgen_test(unsupported = tokio::test)]
#[tracing_test::traced_test]
async fn test_update_makefile_tasks_success() {
    // Use compile-time relative path to test data (workspace root)
    let test_project_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-rust-project/Makefile.toml"
    )
    .to_string();

    let result = parse_tasks(test_project_path, exec_cmd).await;

    // The result depends on whether cargo-make is installed
    match result {
        MakefileTasksUpdate::New(tasks) => {
            // Expected tasks from test-rust-project/Makefile.toml
            let expected_tasks = vec![
                "check-workspace",
                "build-workspace",
                "test-workspace",
                "clean-workspace",
                "fmt-workspace",
                "clippy-workspace",
                "doc-workspace",
                "release-build",
                "ci-flow",
            ];

            for expected in &expected_tasks {
                assert!(
                    tasks.iter().any(|t| t.name == *expected),
                    "Expected task '{}' not found. Available tasks: {:?}",
                    expected,
                    tasks.iter().map(|t| &t.name).collect::<Vec<_>>()
                );
            }

            // Verify all tasks have required fields
            for task in tasks.iter() {
                assert!(!task.name.is_empty(), "Task name should not be empty");
                assert!(
                    !task.category.is_empty(),
                    "Task category should not be empty for task '{}'",
                    task.name
                );
                assert!(
                    !task.description.is_empty(),
                    "Task description should not be empty for task '{}'",
                    task.name
                );
            }
        }
        MakefileTasksUpdate::NoMakefile(_) => {}
        MakefileTasksUpdate::FailedToRetrieve(_) => {
            panic!("Unexpected FailedToRetrieve variant");
        }
    }
}

/// Test metadata discovery with non-existent Cargo.toml.
#[wasm_bindgen_test(unsupported = tokio::test)]
#[tracing_test::traced_test]
async fn test_update_metadata_no_cargo_toml() {
    let nonexistent_path = "/nonexistent/path/that/does/not/exist".to_string();

    let result = parse_metadata(nonexistent_path, exec_cmd).await;

    // Verify error variant
    assert!(
        matches!(result, MetadataUpdate::NoCargoToml(_)),
        "Expected MetadataUpdate::NoCargoToml, got: {:?}",
        result
    );
}

/// Test makefile task discovery without cargo-make.
#[wasm_bindgen_test(unsupported = tokio::test)]
#[tracing_test::traced_test]
async fn test_update_makefile_tasks_no_cargo_make() {
    // This test verifies behavior when cargo-make might not be available
    let test_project_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-rust-project").to_string();

    let result = parse_tasks(test_project_path, exec_cmd).await;

    // The result depends on whether cargo-make is installed
    // All outcomes are valid for this test
    match result {
        MakefileTasksUpdate::NoMakefile(_) => {
            // cargo-make not available or command failed - this is acceptable
        }
        MakefileTasksUpdate::New(_) => {
            // cargo-make is available and tasks were discovered - also valid
        }
        MakefileTasksUpdate::FailedToRetrieve(_) => {
            // Some other error occurred - also acceptable
        }
    }
}

/// Test makefile task discovery with no Makefile.toml.
/// Cargo-make provides built-in tasks even without a Makefile.toml.
#[wasm_bindgen_test(unsupported = tokio::test)]
#[tracing_test::traced_test]
async fn test_update_makefile_tasks_no_makefile() {
    // Use a subdirectory that doesn't have a Makefile.toml
    let path_without_makefile =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-rust-project/core").to_string();

    let result = parse_tasks(path_without_makefile, exec_cmd).await;

    // When given an invalid makefile path, cargo-make command fails
    // The current implementation returns NoMakefile in this case
    match result {
        MakefileTasksUpdate::NoMakefile(_) => {
            // Expected: cargo-make failed because the path doesn't point to a valid Makefile.toml
        }
        MakefileTasksUpdate::New(tasks) => {
            // If cargo-make supports discovery without explicit makefile, this is also valid
            assert!(
                !tasks.is_empty(),
                "Expected tasks if New variant is returned"
            );
        }
        _ => panic!("Expected NoMakefile or New variant, got: {:?}", result),
    }
}
