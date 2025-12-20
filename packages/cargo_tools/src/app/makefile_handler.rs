use std::sync::Arc;

use futures::StreamExt;
use iced_headless::{Subscription, Task};

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
    New(Arc<MakefileTasks>),
    NoMakefile,
    FailedToRetrieve,
}

pub enum MakefileHandlerMessage {
    RootDirChanged(String),
    MakefileChanged,
    MakefileTasksUpdate(MakefileTasksUpdate),
}
use MakefileHandlerMessage as Msg;

pub struct MakefileHandler {
    makefile: String,
}

impl MakefileHandler {
    pub fn update<RuntimeT: Runtime>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirChanged(root_dir) => {
                self.makefile = format!("{root_dir}/Makefile.toml");
                Task::future(update_makefile_tasks::<RuntimeT>(self.makefile.clone()))
                    .map(Msg::MakefileTasksUpdate)
            }
            Msg::MakefileChanged => {
                Task::future(update_makefile_tasks::<RuntimeT>(self.makefile.clone()))
                    .map(Msg::MakefileTasksUpdate)
            }
            Msg::MakefileTasksUpdate(_) => {
                let makefile = self.makefile.clone();
                Task::future(async move {
                    RuntimeT::file_changed_notifier(makefile).next().await;
                })
                .map(|()| Msg::MakefileChanged)
            }
        }
    }

    pub fn subscription<RuntimeT: Runtime>(&self) -> Subscription<Msg> {
        Subscription::run(RuntimeT::current_dir_notitifier).map(Msg::RootDirChanged)
    }
}

pub async fn update_makefile_tasks<RuntimeT: Runtime>(makefile: String) -> MakefileTasksUpdate {
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
        "cargo make --list-all-steps --makefile {makefile} --output-format markdown-single-page"
    ))
    .await
    {
        Ok(output) => parse_makefile_output::<RuntimeT>(&output).await,
        Err(e) => {
            RuntimeT::log(format!("Failed to list cargo-make tasks: {e}")).await;
            MakefileTasksUpdate::NoMakefile
        }
    }
}

/// Parse cargo-make task list output into structured task data
async fn parse_makefile_output<RuntimeT: Runtime>(output: &str) -> MakefileTasksUpdate {
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
        } else if !line.is_empty() {
            if let Some(task) = tasks.last_mut() {
                task.description.push('\n');
                task.description.push_str(line);
            }
        }
    }

    RuntimeT::log(format!("Discovered {} cargo-make tasks", tasks.len())).await;
    MakefileTasksUpdate::New(Arc::new(tasks))
}
