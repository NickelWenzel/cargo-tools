use cargo_tools::{
    cargo_make::{MakefileTask, MakefileTasks},
    xtask::PinnedAlias,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    extension::tasks::cargo_make::tree_provider::CargoMakeNodeHandler,
    icon::{MAKEFILE_TASK, XTASK_ALIAS},
};

#[wasm_bindgen(raw_module = "../cargoMakeTreeProvider.ts")]
extern "C" {
    pub type CargoMakePinnedNode;

    #[wasm_bindgen(constructor)]
    fn new(
        label: String,
        icon: crate::icon::Icon,
        collapsible_state: u32,
        context_value: String,
        description: String,
        tooltip: String,
        handler: CargoMakeNodeHandler,
    ) -> CargoMakePinnedNode;

    pub type PinnedAliasNode;

    #[wasm_bindgen(constructor)]
    fn new(
        label: String,
        icon: crate::icon::Icon,
        collapsible_state: u32,
        context_value: String,
        description: String,
        tooltip: String,
    ) -> PinnedAliasNode;
}

const PINNED_CONTEXT: &str = "pinned-task";
const PINNED_ALIAS_CONTEXT: &str = "pinned-alias";

// Make sure to keep this up to date with 'TreeItemCollapsibleState'
enum CollapsibleState {
    None = 0,
}

/// The handler implementing for the vs code tree item bodies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct CargoMakePinnedTreeProviderHandler {
    pinned_tasks: MakefileTasks,
    pinned_aliases: Vec<PinnedAlias>,
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

    #[wasm_bindgen]
    pub fn pinned_aliases(&self) -> Vec<PinnedAliasNode> {
        self.pinned_aliases
            .iter()
            .cloned()
            .map(pinned_alias_node)
            .collect()
    }
}

/// Methods not exported to typescript
impl CargoMakePinnedTreeProviderHandler {
    pub fn new(pinned_tasks: MakefileTasks, pinned_aliases: Vec<PinnedAlias>) -> Self {
        Self {
            pinned_tasks,
            pinned_aliases,
        }
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

fn pinned_alias_node(alias: PinnedAlias) -> PinnedAliasNode {
    let label = alias.name.clone();
    let collapsible_state = CollapsibleState::None as u32;
    let description = alias.extra_args.join(" ");
    let tooltip = if description.is_empty() {
        format!("Alias: {label}")
    } else {
        format!("Alias: {label} {description}")
    };

    PinnedAliasNode::new(
        label,
        XTASK_ALIAS,
        collapsible_state,
        PINNED_ALIAS_CONTEXT.to_string(),
        description,
        tooltip,
    )
}
