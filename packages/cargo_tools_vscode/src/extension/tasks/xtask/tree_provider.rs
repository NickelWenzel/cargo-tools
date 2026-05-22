use cargo_tools::xtask::XtaskAliases;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{icon::XTASK_ALIAS, vs_code_api::XtaskNode};

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
            .cloned()
            .map(|alias| {
                let label = alias.name.clone();
                let description = alias.command_display();
                let tooltip = format!("cargo {label}");
                XtaskNode::new(
                    label,
                    XTASK_ALIAS,
                    CollapsibleState::None as u32,
                    ALIAS_CONTEXT.to_string(),
                    description,
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
