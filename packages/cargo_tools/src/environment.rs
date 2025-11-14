use std::sync::{Arc, RwLock};

use cargo_metadata::{Metadata, MetadataCommand};

use crate::runtime::Runtime;

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Arc<RwLock<Metadata>>),
    NoCargoToml,
    FailedToRetrieve,
}

fn spawn_manifest_handler<RuntimeT: Runtime>(
    metadata_tx: async_broadcast::Sender<Arc<RwLock<Metadata>>>,
    mut manifest_dir_rx: async_broadcast::Receiver<String>,
) -> RuntimeT::ThreadHandle {
    RuntimeT::spawn(async move {
        while let Some(manifest_dir) = manifest_dir_rx.next().await {
            match update_metadata(&manifest_dir).await {
                MetadataUpdate::New(metadata) => {
                    let _ = metadata_tx.broadcast(metadata).await;
                }
                MetadataUpdate::NoCargoToml | MetadataUpdate::FailedToRetrieve => {}
            }
        }
    })
}

async fn update_metadata<RuntimeT: Runtime>(manifest_dir: &str) -> MetadataUpdate {
    // Construct cargo metadata command with manifest path
    let command =
        format!("cargo metadata --format-version 1 --manifest-path {manifest_dir}/Cargo.toml");

    // Execute command via runtime
    let Ok(metadata) = RuntimeT::exec(command)
        .await
        .inspect_err(|e| RuntimeT::log(format!("Failed to generate cargo metadata: {e}")))
    else {
        return MetadataUpdate::NoCargoToml;
    };

    // Convert JsString to Rust String
    let Some(metadata) = metadata.lines().find(|line| line.starts_with('{')) else {
        RuntimeT::log("Cargo metadata do not contain valid JSON".to_string()).await;
        return MetadataUpdate::FailedToRetrieve;
    };

    // Parse JSON output into Metadata
    let Ok(metadata) = MetadataCommand::parse(metadata)
        .inspect_err(|e| RuntimeT::log(format!("Failed to parse cargo metadata: {e}")))
    else {
        return MetadataUpdate::NoCargoToml;
    };

    MetadataUpdate::New(Arc::new(RwLock::new(metadata)))
}

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

fn spawn_makefile_handler<RuntimeT: Runtime>(
    makefile_tasks_tx: async_broadcast::Sender<Arc<RwLock<MakefileTasks>>>,
    mut workspace_root_rx: async_broadcast::Receiver<String>,
) -> RuntimeT::ThreadHandle {
    RuntimeT::spawn(async move {
        while let Some(workspace_root) = workspace_root_rx.next().await {
            match update_makefile_tasks::<RuntimeT>(&workspace_root).await {
                MakefileTasksUpdate::New(makefile_tasks) => {
                    let _ = makefile_tasks_tx.broadcast(makefile_tasks).await;
                }
                MakefileTasksUpdate::NoMakefile | MakefileTasksUpdate::FailedToRetrieve => {}
            }
        }
    })
}

/// Parse cargo-make task list output into structured task data
fn parse_makefile_output(output: &str) -> Vec<MakefileTask> {
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

    tasks
}

async fn update_makefile_tasks<RuntimeT: Runtime>(workspace_root: &str) -> MakefileTasksUpdate {
    // Check if cargo-make is available
    if RuntimeT::exec("cargo make --version".to_string())
        .await
        .is_err()
    {
        RuntimeT::log("cargo-make not available, skipping task discovery".to_string()).await;
        return MakefileTasksUpdate::NoMakefile;
    }

    // Execute cargo-make to list all tasks
    let Ok(output) = RuntimeT::exec("cargo make --list-all-steps".to_string())
        .await
        .inspect_err(|e| RuntimeT::log(format!("Failed to list cargo-make tasks: {e}")))
    else {
        return MakefileTasksUpdate::FailedToRetrieve;
    };

    // Parse the output
    let tasks = parse_makefile_output(&output);

    if tasks.is_empty() {
        RuntimeT::log("No cargo-make tasks found".to_string()).await;
        return MakefileTasksUpdate::NoMakefile;
    }

    RuntimeT::log(format!("Discovered {} cargo-make tasks", tasks.len())).await;
    MakefileTasksUpdate::New(Arc::new(RwLock::new(tasks)))
}

pub fn spawn_environment<RuntimeT: Runtime>(
    metadata_tx: async_broadcast::Sender<Arc<RwLock<Metadata>>>,
    makefile_tasks_tx: async_broadcast::Sender<Arc<RwLock<MakefileTasks>>>,
) -> EnvironmentHandles<RuntimeT> {
    let workspace_root_rx = RuntimeT::current_dir_notitifier();
    let manifest_handle = spawn_manifest_handler(metadata_tx, workspace_root_rx.clone());
    let makefile_handle = spawn_makefile_handler(makefile_tasks_tx, workspace_root_rx);

    EnvironmentHandles {
        manifest_handle,
        makefile_handle,
    }
}

pub struct EnvironmentHandles<RuntimeT: Runtime> {
    pub manifest_handle: RuntimeT::ThreadHandle,
    pub makefile_handle: RuntimeT::ThreadHandle,
}
