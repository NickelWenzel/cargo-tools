//! VS Code extension contribution point types.
//!
//! This module provides type-safe representations of VS Code extension contribution points
//! that can be serialized to JSON matching the structure expected in `package.json`.
//!
//! # Example
//!
//! Creating individual contribution types:
//!
//! ```
//! use cargo_tools::contributes::{Command, View, Keybinding};
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
//! // Create a view
//! let view = View {
//!     id: "myView".to_string(),
//!     name: "My View".to_string(),
//!     icon: "$(package)".to_string(),
//!     when: Some("myExtension:active".to_string()),
//! };
//!
//! // Create a keybinding
//! let keybinding = Keybinding {
//!     command: "my.command".to_string(),
//!     key: "ctrl+shift+p".to_string(),
//!     when: Some("editorTextFocus".to_string()),
//! };
//! ```
//!
//! Accessing the static contributes data:
//!
//! ```
//! use cargo_tools::contributes::data::CONTRIBUTES;
//!
//! // Access the static contributes instance
//! let json = serde_json::to_string_pretty(&*CONTRIBUTES).unwrap();
//! println!("Extension contributes: {}", json);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use command::Command;
pub use configuration::{ConfigPropertyType, Configuration, ConfigurationProperty};
pub use keybinding::Keybinding;
pub use menu::{MenuItem, Menus};
pub use task_definition::{TaskDefinition, TaskProperty};
pub use view::{View, ViewContainer, Views, ViewsContainers};

/// Top-level contribution points for a VS Code extension.
///
/// This struct represents the complete `contributes` field in package.json and can be
/// serialized to JSON matching the VS Code extension manifest format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contributes {
    pub commands: Vec<Command>,
    #[serde(rename = "viewsContainers")]
    pub views_containers: ViewsContainers,
    pub views: Views,
    pub menus: Menus,
    pub configuration: Configuration,
    #[serde(rename = "taskDefinitions")]
    pub task_definitions: Vec<TaskDefinition>,
    pub keybindings: Vec<Keybinding>,
}

mod command {
    use super::*;

    /// A VS Code command contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Command {
        pub command: String,
        pub title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub category: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enablement: Option<String>,
    }
}

mod view {
    use super::*;

    /// Container for view definitions organized by location.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Views {
        pub explorer: Vec<View>,
        #[serde(rename = "cargoTools")]
        pub cargo_tools: Vec<View>,
    }

    /// A VS Code view contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct View {
        pub id: String,
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub when: Option<String>,
        pub icon: String,
    }

    /// Container for view containers organized by location.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ViewsContainers {
        pub activitybar: Vec<ViewContainer>,
    }

    /// A VS Code view container contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ViewContainer {
        pub id: String,
        pub title: String,
        pub icon: String,
    }
}

mod menu {
    use super::*;

    /// Container for menu contributions organized by location.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Menus {
        #[serde(rename = "view/title")]
        pub view_title: Vec<MenuItem>,
        #[serde(rename = "view/item/context")]
        pub view_item_context: Vec<MenuItem>,
        #[serde(rename = "commandPalette")]
        pub command_palette: Vec<MenuItem>,
    }

    /// A menu item contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct MenuItem {
        pub command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub when: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub group: Option<String>,
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
        pub description: String,
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

mod task_definition {
    use super::*;

    /// A task definition contribution.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TaskDefinition {
        #[serde(rename = "type")]
        pub type_: String,
        pub required: Vec<String>,
        pub properties: HashMap<String, TaskProperty>,
    }

    /// A task property definition.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TaskProperty {
        #[serde(rename = "type")]
        pub type_: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub items: Option<Box<TaskProperty>>,
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

/// Static data module containing factory functions for creating contributes instances.
pub mod data {
    use super::*;
    use once_cell::sync::Lazy;

    /// Static CONTRIBUTES instance containing all extension contribution points.
    ///
    /// This lazy-initialized static variable provides access to the complete set of
    /// contribution points defined for the cargo-tools VS Code extension.
    pub static CONTRIBUTES: Lazy<Contributes> = Lazy::new(|| Contributes {
        commands: all_commands(),
        views_containers: all_views_containers(),
        views: all_views(),
        menus: all_menus(),
        configuration: extension_configuration(),
        task_definitions: all_task_definitions(),
        keybindings: all_keybindings(),
    });

    fn all_commands() -> Vec<Command> {
        vec![
            Command {
                command: "cargo-tools.testRust".to_string(),
                title: "Test Rust bindgen".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(gear)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectProfile".to_string(),
                title: "Select Build Profile".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(gear)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectPackage".to_string(),
                title: "Select Package".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(package)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectBuildTarget".to_string(),
                title: "Select Build Target".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(tools)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectRunTarget".to_string(),
                title: "Select Run Target".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(play)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectBenchmarkTarget".to_string(),
                title: "Select Benchmark Target".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(dashboard)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectPlatformTarget".to_string(),
                title: "Select Platform Target".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(desktop-download)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.installPlatformTarget".to_string(),
                title: "Install Platform Target".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(cloud-download)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.setRustAnalyzerCheckTargets".to_string(),
                title: "Set rust-analyzer check targets".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(checklist)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.buildDocs".to_string(),
                title: "Build Documentation".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(book)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.selectFeatures".to_string(),
                title: "Select Features".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(extensions)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.refresh".to_string(),
                title: "Refresh".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(refresh)".to_string()),
                enablement: None,
            },
            Command {
                command: "cargo-tools.clean".to_string(),
                title: "Clean Build Artifacts".to_string(),
                category: Some("Cargo Tools".to_string()),
                icon: Some("$(trash)".to_string()),
                enablement: None,
            },
        ]
    }

    fn all_views_containers() -> ViewsContainers {
        ViewsContainers {
            activitybar: vec![ViewContainer {
                id: "cargoTools".to_string(),
                title: "Cargo Tools".to_string(),
                icon: "$(package)".to_string(),
            }],
        }
    }

    fn all_views() -> Views {
        Views {
            explorer: vec![View {
                id: "cargoToolsExplorer".to_string(),
                name: "Cargo Tools".to_string(),
                icon: "$(package)".to_string(),
                when: Some("cargoTools:workspaceHasCargo".to_string()),
            }],
            cargo_tools: vec![
                View {
                    id: "cargoToolsProjectStatus".to_string(),
                    name: "Project Status".to_string(),
                    icon: "$(package)".to_string(),
                    when: Some("cargoTools:workspaceHasCargo".to_string()),
                },
                View {
                    id: "cargoToolsProjectOutline".to_string(),
                    name: "Project Outline".to_string(),
                    icon: "$(package)".to_string(),
                    when: Some("cargoTools:workspaceHasCargo".to_string()),
                },
                View {
                    id: "cargoToolsMakefile".to_string(),
                    name: "Makefile".to_string(),
                    icon: "$(tools)".to_string(),
                    when: Some(
                        "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile"
                            .to_string(),
                    ),
                },
                View {
                    id: "cargoToolsPinnedMakefileTasks".to_string(),
                    name: "Pinned Makefile Tasks".to_string(),
                    icon: "$(pin)".to_string(),
                    when: Some(
                        "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile"
                            .to_string(),
                    ),
                },
            ],
        }
    }

    fn all_menus() -> Menus {
        Menus {
            view_title: vec![
                MenuItem {
                    command: "cargo-tools.refresh".to_string(),
                    when: Some("view == cargoToolsProjectStatus".to_string()),
                    group: Some("navigation".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.clean".to_string(),
                    when: Some("view == cargoToolsProjectStatus".to_string()),
                    group: Some("navigation".to_string()),
                },
            ],
            view_item_context: vec![],
            command_palette: vec![],
        }
    }

    fn extension_configuration() -> Configuration {
        let mut properties = HashMap::new();
        properties.insert(
            "cargoTools.cargoCommand".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::String,
                default: Some(serde_json::json!("cargo")),
                description: "Command to invoke instead of 'cargo'. This can be a custom wrapper or alternative cargo implementation.".to_string(),
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.useRustAnalyzerEnvAndArgs".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Boolean,
                default: Some(serde_json::json!(false)),
                description: "Use rust-analyzer settings for cargo command and environment."
                    .to_string(),
                items: None,
                additional_properties: None,
            },
        );
        Configuration {
            title: "Cargo Tools".to_string(),
            properties,
        }
    }

    fn all_task_definitions() -> Vec<TaskDefinition> {
        let mut cargo_props = HashMap::new();
        cargo_props.insert(
            "command".to_string(),
            TaskProperty {
                type_: "string".to_string(),
                description: Some("The cargo command to run".to_string()),
                items: None,
            },
        );
        cargo_props.insert(
            "profile".to_string(),
            TaskProperty {
                type_: "string".to_string(),
                description: Some("The build profile to use".to_string()),
                items: None,
            },
        );
        cargo_props.insert(
            "target".to_string(),
            TaskProperty {
                type_: "string".to_string(),
                description: Some("The target to build".to_string()),
                items: None,
            },
        );

        let mut cargo_make_props = HashMap::new();
        cargo_make_props.insert(
            "task".to_string(),
            TaskProperty {
                type_: "string".to_string(),
                description: Some("The cargo-make task to run".to_string()),
                items: None,
            },
        );

        vec![
            TaskDefinition {
                type_: "cargo".to_string(),
                required: vec!["command".to_string()],
                properties: cargo_props,
            },
            TaskDefinition {
                type_: "cargo-make".to_string(),
                required: vec!["task".to_string()],
                properties: cargo_make_props,
            },
        ]
    }

    fn all_keybindings() -> Vec<Keybinding> {
        vec![
            Keybinding {
                command: "cargo-tools.projectStatus.build".to_string(),
                key: "f7".to_string(),
                when: Some("cargoTools:workspaceHasCargo".to_string()),
            },
            Keybinding {
                command: "cargo-tools.projectStatus.run".to_string(),
                key: "ctrl+shift+f5".to_string(),
                when: Some("cargoTools:workspaceHasCargo".to_string()),
            },
            Keybinding {
                command: "cargo-tools.projectStatus.debug".to_string(),
                key: "shift+f5".to_string(),
                when: Some("cargoTools:workspaceHasCargo".to_string()),
            },
        ]
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
            category: Some("Cat".to_string()),
            icon: Some("$(icon)".to_string()),
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
    fn view_serializes_with_correct_field_names() {
        let view = View {
            id: "test.view".to_string(),
            name: "Test View".to_string(),
            icon: "$(package)".to_string(),
            when: Some("test:condition".to_string()),
        };

        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["id"], "test.view");
        assert_eq!(json["name"], "Test View");
        assert_eq!(json["when"], "test:condition");
        assert_eq!(json["icon"], "$(package)");
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
            description: "Test property".to_string(),
            items: None,
            additional_properties: None,
        };

        let json = serde_json::to_value(&prop).unwrap();
        assert_eq!(json["type"], "string");
        assert_eq!(json["default"], "default_value");
        assert_eq!(json["description"], "Test property");
    }

    #[test]
    fn task_definition_with_properties() {
        let mut properties = HashMap::new();
        properties.insert(
            "command".to_string(),
            TaskProperty {
                type_: "string".to_string(),
                description: Some("The cargo command".to_string()),
                items: None,
            },
        );

        let task = TaskDefinition {
            type_: "cargo".to_string(),
            required: vec!["command".to_string()],
            properties,
        };

        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(json["type"], "cargo");
        assert_eq!(json["required"][0], "command");
        assert_eq!(json["properties"]["command"]["type"], "string");
    }

    #[test]
    fn static_contributes_serializes_to_json() {
        let json = serde_json::to_value(&*data::CONTRIBUTES).unwrap();

        assert!(json["commands"].is_array());
        assert!(json["commands"].as_array().unwrap().len() > 0);
        assert!(json["viewsContainers"]["activitybar"].is_array());
        assert!(json["views"]["cargoTools"].is_array());
        assert_eq!(json["configuration"]["title"], "Cargo Tools");
        assert!(json["taskDefinitions"].is_array());
        assert!(json["keybindings"].is_array());
    }

    #[test]
    fn round_trip_serialization() {
        let original = &*data::CONTRIBUTES;
        let json = serde_json::to_string(original).unwrap();
        let deserialized: Contributes = serde_json::from_str(&json).unwrap();

        assert_eq!(&deserialized, original);
    }

    #[test]
    fn command_without_optional_fields_omits_them() {
        let cmd = Command {
            command: "test.cmd".to_string(),
            title: "Test".to_string(),
            category: None,
            icon: None,
            enablement: None,
        };
        let json = serde_json::to_value(&cmd).unwrap();

        assert!(!json.as_object().unwrap().contains_key("category"));
        assert!(!json.as_object().unwrap().contains_key("icon"));
        assert!(!json.as_object().unwrap().contains_key("enablement"));
    }
}
