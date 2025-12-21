//! VS Code extension contribution point base types.
//!
//! This module provides base type-safe representations of VS Code extension contribution points
//! that can be serialized to JSON matching the structure expected in `package.json`.
//!
//! # Example
//!
//! Creating individual contribution types:
//!
//! ```
//! use cargo_tools::contributes::{Command, Icon, Keybinding};
//!
//! // Create a command
//! let cmd = Command {
//!     command: "my.command".to_string(),
//!     title: "My Command".to_string(),
//!     category: "My Extension".to_string(),
//!     icon: Icon::Gear,
//!     enablement: Some("myExtension:enabled".to_string()),
//! };
//!
//! // Create a keybinding
//! let keybinding = Keybinding {
//!     command: "my.command".to_string(),
//!     key: "ctrl+shift+p".to_string(),
//!     when: Some("editorTextFocus".to_string()),
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use command::Command;
pub use configuration::{ConfigPropertyType, Configuration, ConfigurationProperty};
pub use icon::Icon;
pub use keybinding::Keybinding;
pub use menu_group::{MenuGroup, MenuGroupType};
pub use task_type::TaskType;
pub use view_id::ViewId;

mod icon {
    use super::*;

    /// VS Code icon reference for commands, views, and other UI elements.
    ///
    /// Icons use the VS Code codicon format: `$(icon-name)`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Icon {
        #[serde(rename = "$(gear)")]
        Gear,
        #[serde(rename = "$(package)")]
        Package,
        #[serde(rename = "$(tools)")]
        Tools,
        #[serde(rename = "$(play)")]
        Play,
        #[serde(rename = "$(dashboard)")]
        Dashboard,
        #[serde(rename = "$(desktop-download)")]
        DesktopDownload,
        #[serde(rename = "$(cloud-download)")]
        CloudDownload,
        #[serde(rename = "$(checklist)")]
        Checklist,
        #[serde(rename = "$(book)")]
        Book,
        #[serde(rename = "$(extensions)")]
        Extensions,
        #[serde(rename = "$(refresh)")]
        Refresh,
        #[serde(rename = "$(trash)")]
        Trash,
        #[serde(rename = "$(filter)")]
        Filter,
        #[serde(rename = "$(edit)")]
        Edit,
        #[serde(rename = "$(clear-all)")]
        ClearAll,
        #[serde(rename = "$(symbol-class)")]
        SymbolClass,
        #[serde(rename = "$(add)")]
        Add,
        #[serde(rename = "$(remove)")]
        Remove,
        #[serde(rename = "$(pin)")]
        Pin,
        #[serde(rename = "$(debug-alt)")]
        DebugAlt,
        #[serde(rename = "$(beaker)")]
        Beaker,
        #[serde(rename = "$(check)")]
        Check,
        #[serde(rename = "$(close)")]
        Close,
        #[serde(rename = "$(group-by-ref-type)")]
        GroupByRefType,
    }
}

mod task_type {
    use super::*;

    /// Task type for VS Code task definitions.
    ///
    /// Represents the supported task types in the cargo-tools extension.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum TaskType {
        /// Standard Cargo task (build, run, test, etc.)
        Cargo,
        /// cargo-make task
        CargoMake,
    }
}

mod menu_group {
    use serde::de::IntoDeserializer;

    use super::*;

    /// Menu group types for VS Code menu contributions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum MenuGroupType {
        /// Navigation group - typically shown in view title areas
        Navigation,
        /// Inline actions - shown directly in the tree view
        Inline,
        /// Selection-related actions
        Selection,
        /// Build-related actions
        Build,
        /// Run-related actions
        Run,
        /// Benchmark-related actions
        Benchmark,
        /// General actions
        Actions,
        /// Modify/edit actions
        Modify,
        /// Context menu actions
        Context,
    }

    /// Menu group with optional position within the group.
    ///
    /// VS Code menu groups can have positions specified as "groupName@N" where N is the position.
    /// This struct encapsulates that pattern.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MenuGroup {
        pub group_type: MenuGroupType,
        pub position: Option<u32>,
    }

    impl MenuGroup {
        /// Create a new menu group without a position.
        pub fn new(group_type: MenuGroupType) -> Self {
            Self {
                group_type,
                position: None,
            }
        }

        /// Create a new menu group with a position.
        pub fn with_position(group_type: MenuGroupType, position: u32) -> Self {
            Self {
                group_type,
                position: Some(position),
            }
        }

        /// Get the string representation in VS Code menu group format.
        pub fn as_str(&self) -> String {
            let group_type = serde_json::to_string(&self.group_type).unwrap();
            let group_type = &group_type[1..group_type.len() - 1];
            if let Some(pos) = self.position {
                format!("{group_type}@{pos}")
            } else {
                group_type.to_string()
            }
        }
    }

    impl Serialize for MenuGroup {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for MenuGroup {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;

            // Parse "groupName@N" or "groupName"
            if let Some((group_str, pos_str)) = s.split_once('@') {
                let position = pos_str
                    .parse::<u32>()
                    .map_err(|e| serde::de::Error::custom(format!("invalid position: {}", e)))?;

                let group_type = MenuGroupType::deserialize(group_str.into_deserializer())?;
                Ok(MenuGroup {
                    group_type,
                    position: Some(position),
                })
            } else {
                let group_type = MenuGroupType::deserialize(s.into_deserializer())?;
                Ok(MenuGroup {
                    group_type,
                    position: None,
                })
            }
        }
    }
}

pub mod view_id {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum ViewId {
        CargoToolsExplorer,
        CargoToolsProjectStatus,
        CargoToolsProjectOutline,
        CargoToolsMakefile,
        CargoToolsPinnedMakefileTasks,
    }
}

mod command {
    use super::*;

    /// A VS Code command contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Command {
        pub command: String,
        pub title: String,
        pub category: String,
        pub icon: Icon,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enablement: Option<String>,
    }

    impl Command {
        /// Create a new command with the "cargo-tools." prefix automatically added.
        ///
        /// # Arguments
        ///
        /// * `command` - The command identifier without the "cargo-tools." prefix
        /// * `title` - The display title for the command
        /// * `icon` - The icon for the command
        /// * `enablement` - Optional enablement condition
        pub fn new(
            command: impl Into<String>,
            title: impl Into<String>,
            icon: Icon,
            enablement: Option<String>,
        ) -> Self {
            Self {
                command: format!("cargo-tools.{}", command.into()),
                title: title.into(),
                category: "Cargo Tools".to_string(),
                icon,
                enablement,
            }
        }
    }
}

mod configuration {
    use super::*;

    /// Configuration contribution for extension settings.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Configuration {
        pub title: String,
        pub properties: HashMap<String, ConfigurationProperty>,
    }

    /// A configuration property definition.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ConfigurationProperty {
        #[serde(rename = "type")]
        pub type_: ConfigPropertyType,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub items: Option<Box<ConfigurationProperty>>,
        #[serde(
            rename = "additionalProperties",
            skip_serializing_if = "Option::is_none"
        )]
        pub additional_properties: Option<Box<ConfigurationProperty>>,
    }

    /// Type of a configuration property.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ConfigPropertyType {
        String,
        Boolean,
        Array,
        Object,
    }
}

mod keybinding {
    use super::*;

    /// A keybinding contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Keybinding {
        pub command: String,
        pub key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub when: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_serializes_correctly() {
        let cmd = Command {
            command: "test.command".to_string(),
            title: "Test".to_string(),
            category: "Cat".to_string(),
            icon: Icon::Gear,
            enablement: Some("test:condition".to_string()),
        };

        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["command"], "test.command");
        assert_eq!(json["title"], "Test");
        assert_eq!(json["category"], "Cat");
        assert_eq!(json["icon"], "$(gear)");
        assert_eq!(json["enablement"], "test:condition");
    }

    #[test]
    fn keybinding_serializes_correctly() {
        let keybinding = Keybinding {
            command: "test.command".to_string(),
            key: "f7".to_string(),
            when: Some("test:active".to_string()),
        };

        let json = serde_json::to_value(&keybinding).unwrap();
        assert_eq!(json["command"], "test.command");
        assert_eq!(json["key"], "f7");
        assert_eq!(json["when"], "test:active");
    }

    #[test]
    fn configuration_property_with_default() {
        let prop = ConfigurationProperty {
            type_: ConfigPropertyType::String,
            default: Some(serde_json::json!("default_value")),
            description: Some("Test property".to_string()),
            items: None,
            additional_properties: None,
        };

        let json = serde_json::to_value(&prop).unwrap();
        assert_eq!(json["type"], "string");
        assert_eq!(json["default"], "default_value");
        assert_eq!(json["description"], "Test property");
    }

    #[test]
    fn command_serializes_all_fields() {
        let cmd = Command {
            command: "test.cmd".to_string(),
            title: "Test".to_string(),
            category: "Test Category".to_string(),
            icon: Icon::Gear,
            enablement: None,
        };
        let json = serde_json::to_value(&cmd).unwrap();

        assert_eq!(json["command"], "test.cmd");
        assert_eq!(json["title"], "Test");
        assert_eq!(json["category"], "Test Category");
        assert_eq!(json["icon"], "$(gear)");
        assert!(!json.as_object().unwrap().contains_key("enablement"));
    }

    #[test]
    fn command_new_adds_prefix() {
        let cmd = Command::new("myCommand", "My Command", Icon::Gear, None);

        assert_eq!(cmd.command, "cargo-tools.myCommand");
        assert_eq!(cmd.title, "My Command");
        assert_eq!(cmd.category, "Cargo Tools");
        assert_eq!(cmd.icon, Icon::Gear);
        assert_eq!(cmd.enablement, None);
    }

    #[test]
    fn command_new_with_enablement() {
        let cmd = Command::new(
            "conditionalCommand",
            "Conditional",
            Icon::Check,
            Some("myExtension:enabled".to_string()),
        );

        assert_eq!(cmd.command, "cargo-tools.conditionalCommand");
        assert_eq!(cmd.enablement, Some("myExtension:enabled".to_string()));
    }
}
