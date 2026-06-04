use cargo_tools::xtask::XtaskAliases;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::icon::XTASK_ALIAS;

#[wasm_bindgen(
    raw_module = "../../../packages/cargo_tools_vscode/src/extension/tasks/xtask/tree_provider.ts"
)]
extern "C" {
    pub type XtaskNode;

    #[wasm_bindgen(constructor)]
    fn new(
        label: String,
        icon: crate::icon::Icon,
        collapsible_state: u32,
        context_value: String,
        description: String,
        tooltip: String,
    ) -> XtaskNode;
}

const ALIAS_CONTEXT: &str = "xtaskAlias";

// Keep in sync with TreeItemCollapsibleState
enum CollapsibleState {
    None = 0,
}

/// Handler for the xtask tree view — provides the flat alias list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct XtaskTreeProviderHandler {
    aliases: XtaskAliases,
}

#[wasm_bindgen]
impl XtaskTreeProviderHandler {
    #[wasm_bindgen]
    pub fn aliases(&self) -> Vec<XtaskNode> {
        self.aliases
            .iter()
            .map(|alias| {
                let description = alias
                    .command
                    .is_empty()
                    .then_some(format!("\nAlias: cargo {}", alias.command_display()))
                    .unwrap_or_default();
                let tooltip = format!("Task: {}{description}", alias.name);

                XtaskNode::new(
                    alias.name.clone(),
                    XTASK_ALIAS,
                    CollapsibleState::None as u32,
                    ALIAS_CONTEXT.to_string(),
                    alias.command_display(),
                    tooltip,
                )
            })
            .collect()
    }
}

impl XtaskTreeProviderHandler {
    pub fn new(aliases: XtaskAliases) -> Self {
        Self { aliases }
    }
}
