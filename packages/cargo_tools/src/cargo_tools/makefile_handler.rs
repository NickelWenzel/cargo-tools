use std::sync::{Arc, RwLock};

use futures::{SinkExt, Stream, StreamExt};
use iced_headless::{stream, Subscription, Task};

use crate::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct MakefileTask {
    pub name: String,
    pub category: String,
    pub description: String,
}

pub type MakefileTasks = Vec<MakefileTask>;

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(Arc<RwLock<MakefileTasks>>),
    NoMakefile,
    FailedToRetrieve,
}

pub enum MakefileHandlerMessage {
    MakefileTasksUpdate(MakefileTasksUpdate),
}
use MakefileHandlerMessage as Msg;

pub struct MakefileHandler;

impl MakefileHandler {
    pub fn subscription<RuntimeT: Runtime + 'static>(&self) -> Subscription<Msg> {
        Subscription::run(makefile_tasks_update::<RuntimeT>).map(Msg::MakefileTasksUpdate)
    }
}

fn makefile_tasks_update<RuntimeT: Runtime>() -> impl Stream<Item = MakefileTasksUpdate> {
    stream::channel(100, async |mut makefile_tasks_update_tx| {
        let mut manifest_dir_rx = RuntimeT::current_dir_notitifier();
        while let Some(manifest_dir) = manifest_dir_rx.next().await {
            let makefile_tasks_update = update_makefile_tasks::<RuntimeT>(manifest_dir).await;
            if makefile_tasks_update_tx
                .send(makefile_tasks_update)
                .await
                .is_err()
            {
                RuntimeT::log("Failed to notify update metadata update".to_string()).await;
            }
        }
    })
}

pub async fn update_makefile_tasks<RuntimeT: Runtime>(
    workspace_root: String,
) -> MakefileTasksUpdate {
    // Check if cargo-make is available
    if RuntimeT::exec("cargo make --version".to_string())
        .await
        .is_err()
    {
        RuntimeT::log("cargo-make not available, skipping task discovery".to_string()).await;
        return MakefileTasksUpdate::NoMakefile;
    }

    // Execute cargo-make to list all tasks
    match RuntimeT::exec(format!(
        "cargo make --list-all-steps --makefile {workspace_root}/Makefile.toml"
    ))
    .await
    {
        Ok(output) => parse_makefile_output::<RuntimeT>(&output).await,
        Err(e) => {
            RuntimeT::log(format!("Failed to list cargo-make tasks: {e}")).await;
            MakefileTasksUpdate::FailedToRetrieve
        }
    }
}

/// Parse cargo-make task list output into structured task data
async fn parse_makefile_output<RuntimeT: Runtime>(output: &str) -> MakefileTasksUpdate {
    let mut tasks = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut current_category = String::new();
    let mut in_task_section = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and header content
        if trimmed.is_empty()
            || trimmed.contains("cargo make")
            || trimmed.contains("Build File:")
            || trimmed.contains("Task:")
            || trimmed.contains("Profile:")
        {
            continue;
        }

        // Check if this is a category header (followed by dashes)
        if let Some(next_line) = lines.get(i + 1) {
            let next_trimmed = next_line.trim();
            if !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| c.is_alphabetic() || c.is_whitespace())
                && next_trimmed.chars().all(|c| c == '-')
            {
                current_category = trimmed.to_string();
                in_task_section = true;
                continue;
            }
        }

        // Skip separator lines
        if trimmed.chars().all(|c| c == '-') {
            continue;
        }

        // Parse task line: "task-name - Description"
        if in_task_section {
            if let Some((name, description)) = trimmed.split_once(" - ") {
                let name = name.trim();
                let description = description.trim();

                // Filter out internal tasks
                if !name.starts_with("pre-")
                    && !name.starts_with("post-")
                    && !name.starts_with("end-")
                    && !name.starts_with("init-")
                {
                    tasks.push(MakefileTask {
                        name: name.to_string(),
                        category: if current_category.is_empty() {
                            "Other".to_string()
                        } else {
                            current_category.clone()
                        },
                        description: if description.is_empty() {
                            "No description".to_string()
                        } else {
                            description.to_string()
                        },
                    });
                }
            }
        }
    }

    RuntimeT::log(format!("Discovered {} cargo-make tasks", tasks.len())).await;
    MakefileTasksUpdate::New(Arc::new(RwLock::new(tasks)))
}
