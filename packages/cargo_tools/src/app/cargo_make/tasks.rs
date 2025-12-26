use crate::runtime::Runtime;

#[derive(Debug, Clone, PartialEq)]
pub struct MakefileTask {
    pub name: String,
    pub category: String,
    pub description: String,
}

pub type MakefileTasks = Vec<MakefileTask>;

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(MakefileTasks),
    NoMakefile,
    FailedToRetrieve,
}

pub async fn parse_tasks<RT: Runtime>(makefile: String) -> MakefileTasksUpdate {
    // Check if cargo-make is available
    if RT::exec("cargo make --version".to_string()).await.is_err() {
        RT::log("cargo-make not available, skipping task discovery".to_string()).await;
        return MakefileTasksUpdate::NoMakefile;
    }

    // Execute cargo-make to list all tasks
    match RT::exec(format!(
        "cargo make --list-all-steps --makefile {makefile} --output-format markdown-single-page"
    ))
    .await
    {
        Ok(output) => parse_makefile_output::<RT>(&output).await,
        Err(e) => {
            RT::log(format!("Failed to list cargo-make tasks: {e}")).await;
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
        } else if !line.is_empty() {
            if let Some(task) = tasks.last_mut() {
                task.description.push('\n');
                task.description.push_str(line);
            }
        }
    }

    RT::log(format!("Discovered {} cargo-make tasks", tasks.len())).await;
    MakefileTasksUpdate::New(tasks)
}
