use std::collections::HashMap;

use cargo_tools::cargo_make::{MakefileTask, MakefileTasks};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    icon::{MAKEFILE_CATEGORY, MAKEFILE_TASK},
    vs_code_api::CargoMakeNode,
};

const TASK_CONTEXT: &str = "makefileTask";
const CATEGORY_CONTEXT: &str = "category";

/// The data  for the vs code tree item bodies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct CargoMakeNodeHandler(CargoMakeNodeInner);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CargoMakeNodeInner {
    Category {
        category: String,
        tasks: MakefileTasks,
    },
    Task {
        task: MakefileTask,
    },
}

#[wasm_bindgen]
impl CargoMakeNodeHandler {
    #[wasm_bindgen]
    pub fn tasks(&self) -> Vec<CargoMakeNode> {
        use CargoMakeNodeInner::*;

        match &self.0 {
            Category { category: _, tasks } => tasks
                .iter()
                .cloned()
                .sorted_by(|t1, t2| t1.name.cmp(&t2.name))
                .map(cargo_make_node_from_task)
                .collect(),
            Task { task: _ } => vec![],
        }
    }
}

fn cargo_make_node_from_task(task: MakefileTask) -> CargoMakeNode {
    let label = task.name.clone();
    let collapsible_state = CollapsibleState::None as u32;
    let description = task.description.clone();
    let tooltip = Some(format!(
        "Task: {label}{}",
        if description.is_empty() {
            String::new()
        } else {
            format!("\n{description}")
        }
    ));
    let handler = CargoMakeNodeHandler::task(task);
    CargoMakeNode::new(
        label,
        MAKEFILE_TASK,
        collapsible_state,
        TASK_CONTEXT.to_string(),
        description,
        handler,
        tooltip,
    )
}

impl CargoMakeNodeHandler {
    fn category(category: String, tasks: MakefileTasks) -> Self {
        Self(CargoMakeNodeInner::Category { category, tasks })
    }

    pub fn task(task: MakefileTask) -> Self {
        Self(CargoMakeNodeInner::Task { task })
    }

    pub fn try_into_task(self) -> Option<MakefileTask> {
        match self.0 {
            CargoMakeNodeInner::Category {
                category: _,
                tasks: _,
            } => None,
            CargoMakeNodeInner::Task { task } => Some(task),
        }
    }
}

/// The handler implementing for the vs code tree item bodies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct CargoMakeTreeProviderHandler {
    tasks: MakefileTasks,
}

/// Methods exported to typescript
#[wasm_bindgen]
impl CargoMakeTreeProviderHandler {
    #[wasm_bindgen]
    pub fn categories(&self) -> Vec<CargoMakeNode> {
        self.tasks
            .iter()
            .fold(HashMap::new(), insert)
            .into_iter()
            .sorted_by(|(c1, _), (c2, _)| c1.cmp(c2))
            .map(from_category)
            .collect()
    }
}

fn insert(
    mut nodes: HashMap<String, Vec<MakefileTask>>,
    task: &MakefileTask,
) -> HashMap<String, Vec<MakefileTask>> {
    nodes
        .entry(task.category.clone())
        .or_default()
        .push(task.clone());
    nodes
}

fn from_category((category, tasks): (String, Vec<MakefileTask>)) -> CargoMakeNode {
    let label = category.clone();
    let collapsible_state = CollapsibleState::Expanded as u32;
    let description = format!("{} tasks", tasks.len());
    let tooltip = None;
    let handler = CargoMakeNodeHandler::category(category, MakefileTasks::from(tasks));
    CargoMakeNode::new(
        label,
        MAKEFILE_CATEGORY,
        collapsible_state,
        CATEGORY_CONTEXT.to_string(),
        description,
        handler,
        tooltip,
    )
}

/// Methods not exported to typescript
impl CargoMakeTreeProviderHandler {
    pub fn new(tasks: MakefileTasks) -> Self {
        Self { tasks }
    }
}

// Make sure to keep this up to date with 'TreeItemCollapsibleState'
enum CollapsibleState {
    None = 0,
    Expanded = 2,
}
