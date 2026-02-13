use std::collections::HashMap;

use cargo_tools::cargo_make::tasks::{MakefileTask, MakefileTasks};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::vs_code_api::CargoMakeNode;

/// The data  for the vs code tree item bodes
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
                .map(|task| {
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
                    CargoMakeNode::new(label, collapsible_state, description, tooltip, handler)
                })
                .collect(),
            Task { task: _ } => vec![],
        }
    }
}

impl CargoMakeNodeHandler {
    pub fn category(category: String, tasks: MakefileTasks) -> Self {
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

/// The handler implementing for the vs code tree item bodes
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
            .fold(HashMap::new(), |mut nodes, task| {
                nodes
                    .entry(task.category.clone())
                    .or_insert_with(Vec::new)
                    .push(task.clone());
                nodes
            })
            .into_iter()
            .map(|(category, tasks)| {
                let label = category.clone();
                let collapsible_state = CollapsibleState::Expanded as u32;
                let description = format!("{} tasks", tasks.len());
                let tooltip = None;
                let handler = CargoMakeNodeHandler::category(category, MakefileTasks::from(tasks));
                CargoMakeNode::new(label, collapsible_state, description, tooltip, handler)
            })
            .collect()
    }
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
