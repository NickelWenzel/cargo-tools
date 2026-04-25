use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    environment::Environment,
    runtime::{CargoTask, Runtime, Task},
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

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(MakefileTasks),
    NoMakefile,
    FailedToRetrieve,
}

pub async fn parse_tasks<RT: Runtime>(makefile: String) -> MakefileTasksUpdate {
    // Check if cargo-make is available
    let cargo = "cargo".to_string();
    let args = vec!["make".to_string(), "--version".to_string()];
    if RT::exec(cargo.clone(), args).await.is_err() {
        RT::log("cargo-make not available, skipping task discovery".to_string());
        return MakefileTasksUpdate::NoMakefile;
    }

    // Execute cargo-make to list all tasks
    let args = vec![
        "make".to_string(),
        "--list-all-steps".to_string(),
        "--makefile".to_string(),
        makefile,
        "--output-format".to_string(),
        "markdown-single-page".to_string(),
    ];
    match RT::exec(cargo, args).await {
        Ok(output) => parse_makefile_output::<RT>(&output).await,
        Err(e) => {
            RT::log(format!("Failed to list cargo-make tasks: {e}"));
            MakefileTasksUpdate::NoMakefile
        }
    }
}

/// Parse cargo-make task list output into structured task data
async fn parse_makefile_output<RT: Runtime>(output: &str) -> MakefileTasksUpdate {
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

    RT::log(format!("Discovered {} cargo-make tasks", tasks.len()));
    MakefileTasksUpdate::New(MakefileTasks(tasks))
}
