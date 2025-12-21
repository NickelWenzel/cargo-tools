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
pub use keybinding::Keybinding;

mod command {
    use super::*;

    /// A VS Code command contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Command {
        pub command: String,
        pub title: String,
        pub category: String,
        pub icon: String,
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
        /// * `icon` - The icon identifier (e.g., "$(gear)")
        /// * `enablement` - Optional enablement condition
        pub fn new(
            command: impl Into<String>,
            title: impl Into<String>,
            icon: impl Into<String>,
            enablement: Option<String>,
        ) -> Self {
            Self {
                command: format!("cargo-tools.{}", command.into()),
                title: title.into(),
                category: "Cargo Tools".to_string(),
                icon: icon.into(),
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
            icon: "$(icon)".to_string(),
            enablement: Some("test:condition".to_string()),
        };

        let json = serde_json::to_value(&cmd).unwrap();
        assert_eq!(json["command"], "test.command");
        assert_eq!(json["title"], "Test");
        assert_eq!(json["category"], "Cat");
        assert_eq!(json["icon"], "$(icon)");
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
            icon: "$(icon)".to_string(),
            enablement: None,
        };
        let json = serde_json::to_value(&cmd).unwrap();

        assert_eq!(json["command"], "test.cmd");
        assert_eq!(json["title"], "Test");
        assert_eq!(json["category"], "Test Category");
        assert_eq!(json["icon"], "$(icon)");
        assert!(!json.as_object().unwrap().contains_key("enablement"));
    }

    #[test]
    fn command_new_adds_prefix() {
        let cmd = Command::new("myCommand", "My Command", "$(gear)", None);
        
        assert_eq!(cmd.command, "cargo-tools.myCommand");
        assert_eq!(cmd.title, "My Command");
        assert_eq!(cmd.category, "Cargo Tools");
        assert_eq!(cmd.icon, "$(gear)");
        assert_eq!(cmd.enablement, None);
    }

    #[test]
    fn command_new_with_enablement() {
        let cmd = Command::new(
            "conditionalCommand",
            "Conditional",
            "$(check)",
            Some("myExtension:enabled".to_string()),
        );
        
        assert_eq!(cmd.command, "cargo-tools.conditionalCommand");
        assert_eq!(cmd.enablement, Some("myExtension:enabled".to_string()));
    }
}
