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
//! use cargo_tools::contributes::{Command, Keybinding};
//!
//! // Create a command
//! let cmd = Command {
//!     command: "my.command".to_string(),
//!     title: "My Command".to_string(),
//!     category: Some("My Extension".to_string()),
//!     icon: Some("$(gear)".to_string()),
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Icon {
        Gear,
        Package,
        Tools,
        Play,
        Dashboard,
        DesktopDownload,
        CloudDownload,
        Checklist,
        Book,
        Extensions,
        Refresh,
        Trash,
        Filter,
        Edit,
        ClearAll,
        SymbolClass,
        Add,
        Remove,
        Pin,
        DebugAlt,
        Beaker,
        Check,
        Close,
        GroupByRefType,
    }

    impl Icon {
        /// Get the icon string in VS Code codicon format.
        pub fn as_str(self) -> &'static str {
            match self {
                Icon::Gear => "$(gear)",
                Icon::Package => "$(package)",
                Icon::Tools => "$(tools)",
                Icon::Play => "$(play)",
                Icon::Dashboard => "$(dashboard)",
                Icon::DesktopDownload => "$(desktop-download)",
                Icon::CloudDownload => "$(cloud-download)",
                Icon::Checklist => "$(checklist)",
                Icon::Book => "$(book)",
                Icon::Extensions => "$(extensions)",
                Icon::Refresh => "$(refresh)",
                Icon::Trash => "$(trash)",
                Icon::Filter => "$(filter)",
                Icon::Edit => "$(edit)",
                Icon::ClearAll => "$(clear-all)",
                Icon::SymbolClass => "$(symbol-class)",
                Icon::Add => "$(add)",
                Icon::Remove => "$(remove)",
                Icon::Pin => "$(pin)",
                Icon::DebugAlt => "$(debug-alt)",
                Icon::Beaker => "$(beaker)",
                Icon::Check => "$(check)",
                Icon::Close => "$(close)",
                Icon::GroupByRefType => "$(group-by-ref-type)",
            }
        }
    }

    impl Serialize for Icon {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for Icon {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            match s.as_str() {
                "$(gear)" => Ok(Icon::Gear),
                "$(package)" => Ok(Icon::Package),
                "$(tools)" => Ok(Icon::Tools),
                "$(play)" => Ok(Icon::Play),
                "$(dashboard)" => Ok(Icon::Dashboard),
                "$(desktop-download)" => Ok(Icon::DesktopDownload),
                "$(cloud-download)" => Ok(Icon::CloudDownload),
                "$(checklist)" => Ok(Icon::Checklist),
                "$(book)" => Ok(Icon::Book),
                "$(extensions)" => Ok(Icon::Extensions),
                "$(refresh)" => Ok(Icon::Refresh),
                "$(trash)" => Ok(Icon::Trash),
                "$(filter)" => Ok(Icon::Filter),
                "$(edit)" => Ok(Icon::Edit),
                "$(clear-all)" => Ok(Icon::ClearAll),
                "$(symbol-class)" => Ok(Icon::SymbolClass),
                "$(add)" => Ok(Icon::Add),
                "$(remove)" => Ok(Icon::Remove),
                "$(pin)" => Ok(Icon::Pin),
                "$(debug-alt)" => Ok(Icon::DebugAlt),
                "$(beaker)" => Ok(Icon::Beaker),
                "$(check)" => Ok(Icon::Check),
                "$(close)" => Ok(Icon::Close),
                "$(group-by-ref-type)" => Ok(Icon::GroupByRefType),
                _ => Err(serde::de::Error::unknown_variant(
                    &s,
                    &[
                        "$(gear)",
                        "$(package)",
                        "$(tools)",
                        "$(play)",
                        "$(dashboard)",
                        "$(desktop-download)",
                        "$(cloud-download)",
                        "$(checklist)",
                        "$(book)",
                        "$(extensions)",
                        "$(refresh)",
                        "$(trash)",
                        "$(filter)",
                        "$(edit)",
                        "$(clear-all)",
                        "$(symbol-class)",
                        "$(add)",
                        "$(remove)",
                        "$(pin)",
                        "$(debug-alt)",
                        "$(beaker)",
                        "$(check)",
                        "$(close)",
                        "$(group-by-ref-type)",
                    ],
                )),
            }
        }
    }
}

mod task_type {
    use super::*;

    /// Task type for VS Code task definitions.
    ///
    /// Represents the supported task types in the cargo-tools extension.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum TaskType {
        /// Standard Cargo task (build, run, test, etc.)
        Cargo,
        /// cargo-make task
        CargoMake,
    }

    impl TaskType {
        /// Get the task type string for VS Code task definitions.
        pub fn as_str(self) -> &'static str {
            match self {
                TaskType::Cargo => "cargo",
                TaskType::CargoMake => "cargo-make",
            }
        }
    }

    impl Serialize for TaskType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for TaskType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            match s.as_str() {
                "cargo" => Ok(TaskType::Cargo),
                "cargo-make" => Ok(TaskType::CargoMake),
                _ => Err(serde::de::Error::custom(format!(
                    "invalid task type: {}",
                    s
                ))),
            }
        }
    }
}

mod menu_group {
    use super::*;
    use std::fmt;

    /// Menu group types for VS Code menu contributions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    impl MenuGroupType {
        /// Get the group type string for VS Code menus.
        pub fn as_str(self) -> &'static str {
            match self {
                MenuGroupType::Navigation => "navigation",
                MenuGroupType::Inline => "inline",
                MenuGroupType::Selection => "selection",
                MenuGroupType::Build => "build",
                MenuGroupType::Run => "run",
                MenuGroupType::Benchmark => "benchmark",
                MenuGroupType::Actions => "actions",
                MenuGroupType::Modify => "modify",
                MenuGroupType::Context => "context",
            }
        }
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
            if let Some(pos) = self.position {
                format!("{}@{}", self.group_type.as_str(), pos)
            } else {
                self.group_type.as_str().to_string()
            }
        }
    }

    impl fmt::Display for MenuGroup {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.as_str())
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

                let group_type = Self::parse_group_type(group_str)?;
                Ok(MenuGroup {
                    group_type,
                    position: Some(position),
                })
            } else {
                let group_type = Self::parse_group_type(&s)?;
                Ok(MenuGroup {
                    group_type,
                    position: None,
                })
            }
        }
    }

    impl MenuGroup {
        fn parse_group_type<E: serde::de::Error>(s: &str) -> Result<MenuGroupType, E> {
            match s {
                "navigation" => Ok(MenuGroupType::Navigation),
                "inline" => Ok(MenuGroupType::Inline),
                "selection" => Ok(MenuGroupType::Selection),
                "build" => Ok(MenuGroupType::Build),
                "run" => Ok(MenuGroupType::Run),
                "benchmark" => Ok(MenuGroupType::Benchmark),
                "actions" => Ok(MenuGroupType::Actions),
                "modify" => Ok(MenuGroupType::Modify),
                "context" => Ok(MenuGroupType::Context),
                _ => Err(serde::de::Error::custom(format!(
                    "invalid menu group: {}",
                    s
                ))),
            }
        }
    }
}

pub mod view_id {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ViewId {
        CargoToolsExplorer,
        CargoToolsProjectStatus,
        CargoToolsProjectOutline,
        CargoToolsMakefile,
        CargoToolsPinnedMakefileTasks,
    }

    impl ViewId {
        pub fn as_str(&self) -> &'static str {
            match self {
                ViewId::CargoToolsExplorer => "cargoToolsExplorer",
                ViewId::CargoToolsProjectStatus => "cargoToolsProjectStatus",
                ViewId::CargoToolsProjectOutline => "cargoToolsProjectOutline",
                ViewId::CargoToolsMakefile => "cargoToolsMakefile",
                ViewId::CargoToolsPinnedMakefileTasks => "cargoToolsPinnedMakefileTasks",
            }
        }

        pub fn from_str(s: &str) -> Option<Self> {
            match s {
                "cargoToolsExplorer" => Some(ViewId::CargoToolsExplorer),
                "cargoToolsProjectStatus" => Some(ViewId::CargoToolsProjectStatus),
                "cargoToolsProjectOutline" => Some(ViewId::CargoToolsProjectOutline),
                "cargoToolsMakefile" => Some(ViewId::CargoToolsMakefile),
                "cargoToolsPinnedMakefileTasks" => Some(ViewId::CargoToolsPinnedMakefileTasks),
                _ => None,
            }
        }
    }

    impl fmt::Display for ViewId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }

    impl Serialize for ViewId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for ViewId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            ViewId::from_str(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid view ID: {}", s)))
        }
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
