use cargo_tools::{
    app::{
        cargo::command::{BuildSubTarget, RunSubTarget},
        cargo_make::tasks::MakefileTask,
    },
    profile::Profile,
};
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

    #[wasm_bindgen(getter)]
    pub fn label(&self) -> String {
        self.label.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn detail(&self) -> Option<String> {
        self.detail.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn picked(&self) -> Option<bool> {
        self.picked
    }
}

pub trait ToQuickPickItem {
    fn to_item(&self, picked: bool) -> QuickPickItem;
}

impl ToQuickPickItem for Profile {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        QuickPickItem::new(self.get_display_name().to_string())
            .with_detail(self.get_description().to_string())
            .with_picked(picked)
    }
}

impl ToQuickPickItem for String {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        QuickPickItem::new(self.clone()).with_picked(picked)
    }
}

impl ToQuickPickItem for Option<String> {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        match self {
            Some(name) => name.to_item(picked),
            None => QuickPickItem::new("No selection".to_string()).with_picked(picked),
        }
    }
}

impl ToQuickPickItem for Option<BuildSubTarget> {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        let Some(target) = &self else {
            return QuickPickItem::new("No selection".to_string()).with_picked(picked);
        };

        let (name, desc) = match target.clone() {
            BuildSubTarget::Bin(name) => (name, "Binary".to_string()),
            BuildSubTarget::Example(name) => (name, "Example".to_string()),
            BuildSubTarget::Lib(name) => (name, "Library".to_string()),
            BuildSubTarget::Bench(name) => (name, "Benchmark".to_string()),
        };

        QuickPickItem::new(name)
            .with_description(desc)
            .with_picked(picked)
    }
}

impl ToQuickPickItem for Option<RunSubTarget> {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        let Some(target) = &self else {
            return QuickPickItem::new("No selection".to_string()).with_picked(picked);
        };

        let (name, desc) = match target.clone() {
            RunSubTarget::Bin(name) => (name, "Binary".to_string()),
            RunSubTarget::Example(name) => (name, "Example".to_string()),
        };

        QuickPickItem::new(name)
            .with_description(desc)
            .with_picked(picked)
    }
}

impl ToQuickPickItem for MakefileTask {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        let MakefileTask {
            name,
            category,
            description,
        } = self.clone();

        QuickPickItem::new(name)
            .with_description(category)
            .with_detail(description)
            .with_picked(picked)
    }
}
