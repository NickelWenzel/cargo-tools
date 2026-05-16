use cargo_tools::cargo_make::{MakefileTask, MakefileTasks};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    extension::tasks::cargo_make::tree_provider::CargoMakeNodeHandler, icon::MAKEFILE_TASK,
    vs_code_api::CargoMakePinnedNode,
};

const PINNED_CONTEXT: &str = "pinned-task";

// Make sure to keep this up to date with 'TreeItemCollapsibleState'
enum CollapsibleState {
    None = 0,
}

/// The handler implementing for the vs code tree item bodies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct CargoMakePinnedTreeProviderHandler {
    pinned_tasks: MakefileTasks,
}

/// Methods exported to typescript
#[wasm_bindgen]
impl CargoMakePinnedTreeProviderHandler {
    #[wasm_bindgen]
    pub fn pinned_tasks(&self) -> Vec<CargoMakePinnedNode> {
        self.pinned_tasks
            .iter()
            .cloned()
            .map(cargo_make_pinned_node_from_task)
            .collect()
    }
}

/// Methods not exported to typescript
impl CargoMakePinnedTreeProviderHandler {
    pub fn new(pinned_tasks: MakefileTasks) -> Self {
        Self { pinned_tasks }
    }
}

fn cargo_make_pinned_node_from_task(task: MakefileTask) -> CargoMakePinnedNode {
    let label = task.name.clone();
    let collapsible_state = CollapsibleState::None as u32;
    let description = task.description.clone();
    let tooltip = format!(
        "Task: {label}{}",
        if description.is_empty() {
            String::new()
        } else {
            format!("\n{description}")
        }
    );
    let handler = CargoMakeNodeHandler::task(task);

    CargoMakePinnedNode::new(
        label,
        MAKEFILE_TASK,
        collapsible_state,
        PINNED_CONTEXT.to_string(),
        description,
        tooltip,
        handler,
    )
}
