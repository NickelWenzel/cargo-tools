use cargo_tools::profile::Profile;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Represents an item in a VS Code quick pick menu.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct QuickPickItem {
    /// The label to display in the quick pick.
    label: String,
    /// Optional description shown after the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Optional detail text shown below the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    /// Whether this item should be selected by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    picked: Option<bool>,
}

#[wasm_bindgen]
impl QuickPickItem {
    /// Creates a new QuickPickItem with a label.
    pub fn new(label: String) -> Self {
        Self {
            label,
            description: None,
            detail: None,
            picked: None,
        }
    }

    /// Sets the description for this item.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the detail text for this item.
    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Sets whether this item should be picked by default.
    pub fn with_picked(mut self, picked: bool) -> Self {
        self.picked = Some(picked);
        self
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn detail(&self) -> Option<String> {
        self.detail.clone()
    }
}

pub trait ToQuickPickItem {
    fn to_item(&self) -> QuickPickItem;
}

impl ToQuickPickItem for Profile {
    fn to_item(&self) -> QuickPickItem {
        QuickPickItem::new(self.get_display_name().to_string())
            .with_detail(self.get_description().to_string())
    }
}
