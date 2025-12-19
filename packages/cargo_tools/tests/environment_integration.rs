//! Integration tests for environment discovery functionality.
//!
//! These tests verify the metadata and makefile task discovery functionality
//! using the test-rust-project as test data.

mod support;
use cargo_tools::cargo_tools::{
    makefile_handler::{update_makefile_tasks, MakefileTasksUpdate},
    metadata_handler::{update_metadata, MetadataUpdate},
};
use support::TestRuntime;

/// Test successful metadata discovery from test-rust-project.
#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_metadata_success() {
    // Use canonicalized absolute path to avoid working directory issues with cmd_lib
    // Note: update_metadata expects manifest directory, not the full Cargo.toml path
    let base_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = base_path
        .join("../../test-rust-project/Cargo.toml")
        .canonicalize()
        .expect("Failed to canonicalize test project path")
        .to_str()
        .unwrap()
        .to_string();

    let result = update_metadata::<TestRuntime>(manifest).await;

    // Verify success variant
    assert!(
        matches!(result, MetadataUpdate::New(_)),
        "Expected MetadataUpdate::New, got: {:?}",
        result
    );

    // Extract and verify metadata contents
    if let MetadataUpdate::New(metadata) = result {
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

        // Verify no error logs
        assert!(!logs_contain("Failed to generate cargo metadata"));
        assert!(!logs_contain("Failed to parse cargo metadata"));
    }
}

/// Test makefile task discovery - skips if cargo-make not installed.
#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_makefile_tasks_success() {
    // Use compile-time relative path to test data (workspace root)
    let test_project_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-rust-project/Makefile.toml"
    )
    .to_string();

    let result = update_makefile_tasks::<TestRuntime>(test_project_path).await;

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

            // Verify success log message
            assert!(logs_contain("Discovered"));
        }
        MakefileTasksUpdate::NoMakefile => {
            // cargo-make is not installed - this is acceptable in CI/test environments
            assert!(logs_contain("cargo-make not available"));
        }
        MakefileTasksUpdate::FailedToRetrieve => {
            panic!("Unexpected FailedToRetrieve variant");
        }
    }
}

/// Test metadata discovery with non-existent Cargo.toml.
#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_metadata_no_cargo_toml() {
    let nonexistent_path = "/nonexistent/path/that/does/not/exist".to_string();

    let result = update_metadata::<TestRuntime>(nonexistent_path).await;

    // Verify error variant
    assert!(
        matches!(result, MetadataUpdate::NoCargoToml),
        "Expected MetadataUpdate::NoCargoToml, got: {:?}",
        result
    );

    // Verify error log message
    assert!(logs_contain("Failed to generate cargo metadata"));
}

/// Test makefile task discovery without cargo-make.
#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_makefile_tasks_no_cargo_make() {
    // This test relies on cargo-make potentially being unavailable or
    // the version check failing naturally in the test environment
    let test_project_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-rust-project").to_string();

    let result = update_makefile_tasks::<TestRuntime>(test_project_path).await;

    // The result depends on whether cargo-make is installed
    // If installed, this test may pass differently
    match result {
        MakefileTasksUpdate::NoMakefile => {
            // Verify appropriate log message
            assert!(logs_contain("cargo-make not available"));
        }
        MakefileTasksUpdate::New(_) => {
            // cargo-make is available and tasks were discovered
            // This is also a valid outcome
        }
        MakefileTasksUpdate::FailedToRetrieve => {
            // Some other error occurred
        }
    }
}

/// Test makefile task discovery with no Makefile.toml.
/// Cargo-make provides built-in tasks even without a Makefile.toml.
#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_makefile_tasks_no_makefile() {
    // Use a subdirectory that doesn't have a Makefile.toml
    let path_without_makefile =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-rust-project/core").to_string();

    let result = update_makefile_tasks::<TestRuntime>(path_without_makefile).await;

    // Cargo-make provides built-in tasks even without a Makefile.toml,
    // so we expect MakefileTasksUpdate::New with tasks
    match result {
        MakefileTasksUpdate::New(tasks) => {
            assert!(!tasks.is_empty(), "Expected built-in cargo-make tasks");
        }
        _ => panic!(
            "Expected New variant with built-in tasks, got: {:?}",
            result
        ),
    }
}
