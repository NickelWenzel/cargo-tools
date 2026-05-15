use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    environment::Environment,
    task::{CargoTask, Task},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MakefileTask {
    pub name: String,
    pub category: String,
    pub description: String,
}

impl MakefileTask {
    pub fn into_task(task: String, environment: Environment) -> CargoTask {
        let Environment {
            env, cargo_command, ..
        } = environment;
        let mut cmd = cargo_command.split_whitespace().map(String::from);
        let (cmd, mut args) = (cmd.next().unwrap(), cmd.collect::<Vec<_>>());
        args.extend(["make".to_string(), task]);

        CargoTask::CargoMake(Task { cmd, args, env })
    }

    fn keep(&self, task_filter: &str, category_filters: &[String]) -> bool {
        // 1. check if category is filetered out filter category
        if category_filters.contains(&self.category) {
            return false;
        }

        // 2. check if there is a task filter
        if task_filter.is_empty() {
            return true;
        }

        // 3. check task filter
        self.name
            .to_lowercase()
            .contains(&task_filter.to_lowercase())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MakefileTasks(Vec<MakefileTask>);

impl MakefileTasks {
    pub fn filtered(&self, task_filter: &str, category_filters: &[String]) -> Self {
        let tasks = self
            .iter()
            .filter(|task| task.keep(task_filter, category_filters))
            .cloned()
            .collect();
        Self(tasks)
    }
}

impl Deref for MakefileTasks {
    type Target = Vec<MakefileTask>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MakefileTasks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<MakefileTask>> for MakefileTasks {
    fn from(value: Vec<MakefileTask>) -> Self {
        Self(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Cargo make is not installed: {0}")]
    CargoMakeNotInstalled(String),
    #[error("No Makefile.toml present: {0}")]
    NoMakefile(String),
    #[error("Failed to retrieve tasks from Makefile.toml: {0}")]
    FailedToRetrieve(String),
}

pub async fn parse_tasks(
    makefile: String,
    exec: impl AsyncFn(String, Vec<String>) -> Result<String, String>,
) -> Result<MakefileTasks, ParseError> {
    // Check if cargo-make is available
    let cargo = "cargo".to_string();
    let args = vec!["make".to_string(), "--version".to_string()];
    exec(cargo.clone(), args)
        .await
        .map_err(ParseError::CargoMakeNotInstalled)?;

    // Execute cargo-make to list all tasks
    let args = vec![
        "make".to_string(),
        "--list-all-steps".to_string(),
        "--makefile".to_string(),
        makefile,
        "--output-format".to_string(),
        "markdown-single-page".to_string(),
    ];

    exec(cargo, args)
        .await
        .map_err(ParseError::NoMakefile)
        .map(|output| parse_makefile_output(&output))
}

/// Parse cargo-make task list output into structured task data
fn parse_makefile_output(output: &str) -> MakefileTasks {
    let mut tasks = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut current_category = String::new();

    for line in lines.iter() {
        if let Some(line) = line.strip_prefix("## ") {
            current_category = line.to_string();
        } else if let Some(line) = line.strip_prefix("* ") {
            let mut split = line.split(" - ");
            let task = if let (Some(task), Some(desc)) = (split.next(), split.next()) {
                task.strip_prefix("**")
                    .and_then(|task| task.strip_suffix("**"))
                    .map(|task| MakefileTask {
                        name: task.to_string(),
                        category: current_category.clone(),
                        description: desc.to_string(),
                    })
            } else {
                None
            };
            if let Some(task) = task {
                tasks.push(task);
            }
        } else if !line.is_empty()
            && let Some(task) = tasks.last_mut()
        {
            task.description.push('\n');
            task.description.push_str(line);
        }
    }

    MakefileTasks(tasks)
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::parse_makefile_output;

    /// Test makefile task discovery - skips if cargo-make not installed.
    #[wasm_bindgen_test(unsupported = test)]
    #[tracing_test::traced_test]
    fn parse_valid_makefile_output() {
        let makefile_output = include_str!("../res/test-rust-project-cargo-make-steps.md");
        let tasks = parse_makefile_output(makefile_output);

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
}
