use std::collections::HashMap;

use cargo_tools::{process::Process, xtask::XtaskAliases};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{icon::XTASK_ALIAS, runtime::exec_vs_code, vs_code_api::XtaskNode};

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
    pub async fn aliases(&self) -> Vec<XtaskNode> {
        let tooltips = join_all(self.aliases.iter().map(|alias| fetch_tooltip(&alias.name))).await;

        self.aliases
            .iter()
            .zip(tooltips)
            .map(|(alias, tooltip)| {
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

pub(super) async fn fetch_tooltip(label: &str) -> String {
    let process = Process::new(
        "cargo".to_string(),
        vec![label.to_string(), "--help".to_string()],
        HashMap::new(),
    );
    match exec_vs_code(process).await {
        Ok(out) if !out.trim().is_empty() => out,
        _ => format!("cargo {label}"),
    }
}

impl XtaskTreeProviderHandler {
    pub fn new(aliases: XtaskAliases) -> Self {
        Self { aliases }
    }
}
