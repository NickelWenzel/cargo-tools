use std::sync::{Arc, Mutex};

use futures::StreamExt;
use iced_headless::{Subscription, Task};
use wasm_async_trait::wasm_async_trait;

use crate::{app::state::State, context::Context, runtime::Runtime};

#[wasm_async_trait]
pub trait CargoMakeUi {
    async fn update(tasks: Arc<Mutex<MakefileTasks>>, state: Arc<Mutex<State>>);
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
    New(MakefileTasks),
    NoMakefile,
    FailedToRetrieve,
}

pub enum CargoMakeMessage {
    RootDirUpdate(String),
    StateUpdate(State),
    MakefileUpdate,
    MakefileTasksUpdate(MakefileTasksUpdate),
}

use CargoMakeMessage as Msg;

pub struct CargoMake {
    makefile: String,
    tasks: Arc<Mutex<MakefileTasks>>,
    state: Arc<Mutex<State>>,
}

impl CargoMake {
    pub fn update<RT: Runtime, UI: CargoMakeUi, CTX: Context>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT, CTX>(root_dir),
            Msg::MakefileUpdate => {
                Task::future(parse_tasks::<RT>(self.makefile.clone())).map(Msg::MakefileTasksUpdate)
            }
            Msg::MakefileTasksUpdate(tasks_update) => self.update_tasks::<RT, UI>(tasks_update),
            Msg::StateUpdate(state) => {
                *self.state.lock().unwrap() = state;
                self.update_ui::<UI>()
            }
        }
    }

    fn update_root_dir<RT: Runtime, CTX: Context>(&mut self, root_dir: String) -> Task<Msg> {
        self.makefile = format!("{root_dir}/Makefile.toml");
        let update =
            Task::future(parse_tasks::<RT>(self.makefile.clone())).map(Msg::MakefileTasksUpdate);
        let prefix = Task::future(async move {
            CTX::update_prefix(root_dir.clone()).await;
        })
        .discard::<Msg>();
        Task::batch([update, prefix])
    }

    fn update_tasks<RT: Runtime, UI: CargoMakeUi>(
        &mut self,
        tasks_update: MakefileTasksUpdate,
    ) -> Task<Msg> {
        match tasks_update {
            MakefileTasksUpdate::New(tasks) => *self.tasks.lock().unwrap() = tasks,
            MakefileTasksUpdate::NoMakefile => *self.tasks.lock().unwrap() = Vec::new(),
            MakefileTasksUpdate::FailedToRetrieve => {}
        }

        let makefile = self.makefile.clone();
        let makefile = Task::future(async move {
            RT::file_changed_notifier(makefile).next().await;
        })
        .map(|()| Msg::MakefileUpdate);

        let ui = self.update_ui::<UI>();

        Task::batch([makefile, ui])
    }

    fn update_ui<UI: CargoMakeUi>(&self) -> Task<Msg> {
        let (tasks, state) = (self.tasks.clone(), self.state.clone());
        Task::future(UI::update(tasks, state)).discard::<Msg>()
    }

    pub fn subscription<RT: Runtime, CTX: Context>(&self) -> Subscription<Msg> {
        let root_dir = Subscription::run(RT::current_dir_notitifier).map(Msg::RootDirUpdate);
        let state = Subscription::run(CTX::state_receiver).map(Msg::StateUpdate);

        Subscription::batch([root_dir, state])
    }
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
