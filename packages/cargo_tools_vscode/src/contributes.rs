//! VS Code extension contribution point types specific to cargo-tools.
//!
//! This module provides VS Code-specific contribution points for the cargo-tools extension.
//! It builds upon the base types from `cargo_tools::contributes` and adds the complete
//! `Contributes` struct with static data.

use cargo_tools::contributes::{Command, Configuration, Keybinding};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use menu::{MenuItem, Menus};
pub use task_definition::{TaskDefinition, TaskProperty};
pub use view::{View, ViewContainer, Views, ViewsContainers};

/// Top-level contribution points for the cargo-tools VS Code extension.
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

/// Static data module containing the cargo-tools extension contribution points.
pub mod data {
    use super::*;
    use cargo_tools::contributes::{ConfigPropertyType, ConfigurationProperty};

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

    pub fn all_commands() -> Vec<Command> {
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

    pub fn all_views_containers() -> ViewsContainers {
        ViewsContainers {
            activitybar: vec![ViewContainer {
                id: "cargoTools".to_string(),
                title: "Cargo Tools".to_string(),
                icon: "$(package)".to_string(),
            }],
        }
    }

    pub fn all_views() -> Views {
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

    pub fn all_menus() -> Menus {
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

    pub fn extension_configuration() -> Configuration {
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

    pub fn all_task_definitions() -> Vec<TaskDefinition> {
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

    pub fn all_keybindings() -> Vec<Keybinding> {
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
        assert!(!json["commands"].as_array().unwrap().is_empty());
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
    fn menu_serializes_correctly() {
        let menu_item = MenuItem {
            command: "test.command".to_string(),
            when: Some("test:condition".to_string()),
            group: Some("navigation".to_string()),
        };

        let json = serde_json::to_value(&menu_item).unwrap();
        assert_eq!(json["command"], "test.command");
        assert_eq!(json["when"], "test:condition");
        assert_eq!(json["group"], "navigation");
    }

    #[test]
    fn view_container_serializes_correctly() {
        let container = ViewContainer {
            id: "testContainer".to_string(),
            title: "Test Container".to_string(),
            icon: "$(package)".to_string(),
        };

        let json = serde_json::to_value(&container).unwrap();
        assert_eq!(json["id"], "testContainer");
        assert_eq!(json["title"], "Test Container");
        assert_eq!(json["icon"], "$(package)");
    }

    #[test]
    fn contributes_matches_package_json() {
        let package_json = include_str!("../../../package.json");
        let package: serde_json::Value = serde_json::from_str(package_json).unwrap();
        
        let package_contributes: Contributes = serde_json::from_value(package["contributes"].clone())
            .expect("Failed to deserialize contributes from package.json");

        assert_eq!(
            &package_contributes, &*data::CONTRIBUTES,
            "Contributes struct does not match package.json"
        );
    }

    #[test]
    fn configuration_properties_have_required_fields() {
        let config = data::extension_configuration();

        for (key, prop) in &config.properties {
            assert!(
                key.starts_with("cargoTools."),
                "Property key {} should start with 'cargoTools.'",
                key
            );
            assert!(
                !prop.description.is_empty(),
                "Property {} should have a description",
                key
            );
        }
    }

    #[test]
    fn all_commands_have_required_fields() {
        let commands = data::all_commands();

        for command in &commands {
            assert!(
                command.command.starts_with("cargo-tools."),
                "Command {} should start with 'cargo-tools.'",
                command.command
            );
            assert!(
                !command.title.is_empty(),
                "Command {} should have a title",
                command.command
            );
            assert_eq!(
                command.category,
                Some("Cargo Tools".to_string()),
                "Command {} should have category 'Cargo Tools'",
                command.command
            );
            assert!(
                command.icon.is_some(),
                "Command {} should have an icon",
                command.command
            );
        }
    }

    #[test]
    fn all_views_have_consistent_naming() {
        let views = data::all_views();

        for view in &views.explorer {
            assert!(
                view.id.starts_with("cargoTools"),
                "Explorer view {} should start with 'cargoTools'",
                view.id
            );
            assert!(!view.name.is_empty(), "View {} should have a name", view.id);
        }

        for view in &views.cargo_tools {
            assert!(
                view.id.starts_with("cargoTools"),
                "CargoTools view {} should start with 'cargoTools'",
                view.id
            );
            assert!(!view.name.is_empty(), "View {} should have a name", view.id);
        }
    }

    #[test]
    fn task_definitions_have_required_properties() {
        let task_defs = data::all_task_definitions();

        for task_def in &task_defs {
            assert!(
                !task_def.type_.is_empty(),
                "Task definition should have a type"
            );
            assert!(
                !task_def.required.is_empty(),
                "Task definition {} should have required properties",
                task_def.type_
            );

            for required_prop in &task_def.required {
                assert!(
                    task_def.properties.contains_key(required_prop),
                    "Task definition {} should define required property {}",
                    task_def.type_,
                    required_prop
                );
            }
        }
    }

    #[test]
    fn keybindings_have_valid_format() {
        let keybindings = data::all_keybindings();

        for keybinding in &keybindings {
            assert!(
                keybinding.command.starts_with("cargo-tools."),
                "Keybinding command {} should start with 'cargo-tools.'",
                keybinding.command
            );
            assert!(
                !keybinding.key.is_empty(),
                "Keybinding for {} should have a key",
                keybinding.command
            );
            assert!(
                keybinding.when.is_some(),
                "Keybinding for {} should have a when clause",
                keybinding.command
            );
        }
    }

    #[test]
    fn menus_reference_valid_commands() {
        let menus = data::all_menus();
        let commands = data::all_commands();
        let command_ids: Vec<String> = commands.iter().map(|c| c.command.clone()).collect();

        for menu_item in &menus.view_title {
            assert!(
                command_ids.contains(&menu_item.command),
                "Menu item references unknown command: {}",
                menu_item.command
            );
        }
    }

    #[test]
    fn views_containers_activitybar_not_empty() {
        let containers = data::all_views_containers();

        assert!(
            !containers.activitybar.is_empty(),
            "Should have at least one activity bar container"
        );

        for container in &containers.activitybar {
            assert!(!container.id.is_empty(), "Container should have an id");
            assert!(!container.title.is_empty(), "Container should have a title");
            assert!(!container.icon.is_empty(), "Container should have an icon");
        }
    }

    #[test]
    fn task_property_types_are_valid() {
        let task_defs = data::all_task_definitions();
        let valid_types = ["string", "boolean", "number", "array", "object"];

        for task_def in &task_defs {
            for (prop_name, prop) in &task_def.properties {
                assert!(
                    valid_types.contains(&prop.type_.as_str()),
                    "Property {} in task {} has invalid type: {}",
                    prop_name,
                    task_def.type_,
                    prop.type_
                );
            }
        }
    }

    #[test]
    fn contributes_has_all_required_sections() {
        let contributes = &*data::CONTRIBUTES;

        assert!(!contributes.commands.is_empty(), "Should have commands");
        assert!(
            !contributes.views_containers.activitybar.is_empty(),
            "Should have activity bar containers"
        );
        assert!(
            !contributes.views.cargo_tools.is_empty(),
            "Should have cargo tools views"
        );
        assert!(
            !contributes.configuration.properties.is_empty(),
            "Should have configuration properties"
        );
        assert!(
            !contributes.task_definitions.is_empty(),
            "Should have task definitions"
        );
        assert!(
            !contributes.keybindings.is_empty(),
            "Should have keybindings"
        );
    }
}
