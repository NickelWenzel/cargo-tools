//! VS Code extension contribution point types specific to cargo-tools.
//!
//! This module provides VS Code-specific contribution points for the cargo-tools extension.
//! It builds upon the base types from `cargo_tools::contributes` and adds the complete
//! `Contributes` struct with static data.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use configuration::Configuration;
pub use menu::{MenuItem, Menus};
pub use task_definition::{TaskDefinition, TaskProperty};
pub use view::{View, ViewContainer, Views, ViewsContainers};

pub use {
    command::Command,
    configuration::{ConfigPropertyType, ConfigurationProperty},
    icon::Icon,
    keybinding::Keybinding,
    menu_group::MenuGroup,
    menu_group::MenuGroupType,
    task_type::TaskType,
    view_id::ViewId,
};

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
        pub id: ViewId,
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
        pub group: Option<MenuGroup>,
    }

    impl MenuItem {
        /// Create a menu item for a view title.
        pub fn for_view_title(
            command: impl Into<String>,
            view: impl Into<String>,
            group: MenuGroup,
        ) -> Self {
            Self {
                command: command.into(),
                when: Some(format!("view == {}", view.into())),
                group: Some(group),
            }
        }

        /// Create a menu item for a view item context.
        pub fn for_view_item_context(
            command: impl Into<String>,
            view: impl Into<String>,
            view_item: impl Into<String>,
            group: MenuGroup,
        ) -> Self {
            Self {
                command: command.into(),
                when: Some(format!(
                    "view == {} && viewItem == {}",
                    view.into(),
                    view_item.into()
                )),
                group: Some(group),
            }
        }

        /// Create a menu item for view item context with regex matching.
        pub fn for_view_item_context_regex(
            command: impl Into<String>,
            view: impl Into<String>,
            view_item_regex: impl Into<String>,
            group: MenuGroup,
        ) -> Self {
            Self {
                command: command.into(),
                when: Some(format!(
                    "view == {} && viewItem =~ /{}/",
                    view.into(),
                    view_item_regex.into()
                )),
                group: Some(group),
            }
        }

        /// Create a menu item hidden from command palette.
        pub fn hide_from_palette(command: impl Into<String>) -> Self {
            Self {
                command: command.into(),
                when: Some("never".to_string()),
                group: None,
            }
        }
    }
}

mod configuration {
    use wasm_bindgen::prelude::wasm_bindgen;

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
    #[wasm_bindgen]
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
        pub type_: TaskType,
        pub required: Vec<String>,
        pub properties: HashMap<String, TaskProperty>,
    }

    impl TaskDefinition {
        /// Create a Cargo task definition with standard properties.
        pub fn cargo() -> Self {
            let mut properties = HashMap::new();
            properties.insert(
                "command".to_string(),
                TaskProperty::string(Some("The cargo command to run".to_string())),
            );
            properties.insert(
                "profile".to_string(),
                TaskProperty::string(Some("The build profile to use".to_string())),
            );
            properties.insert(
                "target".to_string(),
                TaskProperty::string(Some("The target to build".to_string())),
            );
            properties.insert(
                "features".to_string(),
                TaskProperty::array_of_strings(Some("Features to enable".to_string())),
            );
            properties.insert(
                "allFeatures".to_string(),
                TaskProperty::boolean(Some("Enable all features".to_string())),
            );

            Self {
                type_: TaskType::Cargo,
                required: vec!["command".to_string()],
                properties,
            }
        }

        /// Create a cargo-make task definition.
        pub fn cargo_make() -> Self {
            let mut properties = HashMap::new();
            properties.insert(
                "task".to_string(),
                TaskProperty::string(Some("The cargo-make task to run".to_string())),
            );

            Self {
                type_: TaskType::CargoMake,
                required: vec!["task".to_string()],
                properties,
            }
        }
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

    impl TaskProperty {
        /// Create a string property.
        pub fn string(description: Option<String>) -> Self {
            Self {
                type_: "string".to_string(),
                description,
                items: None,
            }
        }

        /// Create a boolean property.
        pub fn boolean(description: Option<String>) -> Self {
            Self {
                type_: "boolean".to_string(),
                description,
                items: None,
            }
        }

        /// Create an array property with string items.
        pub fn array_of_strings(description: Option<String>) -> Self {
            Self {
                type_: "array".to_string(),
                description,
                items: Some(Box::new(TaskProperty {
                    type_: "string".to_string(),
                    description: None,
                    items: None,
                })),
            }
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

/// Static data module containing the cargo-tools extension contribution points.
pub mod data {
    use super::*;

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
            Command::new("testRust", "Test Rust bindgen", Icon::Gear, None),
            Command::new("selectProfile", "Select Build Profile", Icon::Gear, None),
            Command::new("selectPackage", "Select Package", Icon::Package, None),
            Command::new(
                "selectBuildTarget",
                "Select Build Target",
                Icon::Tools,
                None,
            ),
            Command::new("selectRunTarget", "Select Run Target", Icon::Play, None),
            Command::new(
                "selectBenchmarkTarget",
                "Select Benchmark Target",
                Icon::Dashboard,
                None,
            ),
            Command::new(
                "selectPlatformTarget",
                "Select Platform Target",
                Icon::DesktopDownload,
                None,
            ),
            Command::new(
                "installPlatformTarget",
                "Install Platform Target",
                Icon::CloudDownload,
                None,
            ),
            Command::new(
                "setRustAnalyzerCheckTargets",
                "Set rust-analyzer check targets",
                Icon::Checklist,
                None,
            ),
            Command::new("buildDocs", "Build Documentation", Icon::Book, None),
            Command::new("selectFeatures", "Select Features", Icon::Extensions, None),
            Command::new("refresh", "Refresh", Icon::Refresh, None),
            Command::new("clean", "Clean Build Artifacts", Icon::Trash, None),
            Command::new(
                "makefile.runTask",
                "Run Makefile Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.selectAndRunTask",
                "Run Makefile Task...",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.setTaskFilter",
                "Filter Tasks",
                Icon::Filter,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.editTaskFilter",
                "Edit Task Filter",
                Icon::Edit,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.clearTaskFilter",
                "Clear Task Filter",
                Icon::ClearAll,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.showCategoryFilter",
                "Filter Categories",
                Icon::SymbolClass,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.clearCategoryFilter",
                "Clear Category Filter",
                Icon::ClearAll,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.add",
                "Add Task",
                Icon::Add,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.remove",
                "Remove Task",
                Icon::Remove,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute",
                "Execute Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.pinTask",
                "Pin Task",
                Icon::Pin,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute1",
                "Execute 1st Pinned Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute2",
                "Execute 2nd Pinned Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute3",
                "Execute 3rd Pinned Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute4",
                "Execute 4th Pinned Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute5",
                "Execute 5th Pinned Task",
                Icon::Play,
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new("projectStatus.build", "Build", Icon::Tools, None),
            Command::new("projectStatus.run", "Run", Icon::Play, None),
            Command::new("projectStatus.debug", "Debug", Icon::DebugAlt, None),
            Command::new("projectStatus.test", "Test", Icon::Beaker, None),
            Command::new("projectStatus.bench", "Benchmark", Icon::Dashboard, None),
            Command::new(
                "projectOutline.selectPackage",
                "Select Package",
                Icon::Check,
                None,
            ),
            Command::new(
                "projectOutline.unselectPackage",
                "Unselect Package",
                Icon::Close,
                None,
            ),
            Command::new(
                "projectOutline.setBuildTarget",
                "Set as Build Target",
                Icon::Tools,
                None,
            ),
            Command::new(
                "projectOutline.unsetBuildTarget",
                "Unset Build Target",
                Icon::Close,
                None,
            ),
            Command::new(
                "projectOutline.setRunTarget",
                "Set as Run Target",
                Icon::Play,
                None,
            ),
            Command::new(
                "projectOutline.unsetRunTarget",
                "Unset Run Target",
                Icon::Close,
                None,
            ),
            Command::new(
                "projectOutline.setBenchmarkTarget",
                "Set as Benchmark Target",
                Icon::Dashboard,
                None,
            ),
            Command::new(
                "projectOutline.unsetBenchmarkTarget",
                "Unset Benchmark Target",
                Icon::Close,
                None,
            ),
            Command::new(
                "projectOutline.buildPackage",
                "Build Package",
                Icon::Tools,
                None,
            ),
            Command::new(
                "projectOutline.testPackage",
                "Test Package",
                Icon::Beaker,
                None,
            ),
            Command::new(
                "projectOutline.cleanPackage",
                "Clean Package",
                Icon::Trash,
                None,
            ),
            Command::new(
                "projectOutline.buildWorkspace",
                "Build Workspace",
                Icon::Tools,
                None,
            ),
            Command::new(
                "projectOutline.testWorkspace",
                "Test Workspace",
                Icon::Beaker,
                None,
            ),
            Command::new(
                "projectOutline.cleanWorkspace",
                "Clean Workspace",
                Icon::Trash,
                None,
            ),
            Command::new(
                "projectOutline.buildTarget",
                "Build Target",
                Icon::Tools,
                None,
            ),
            Command::new("projectOutline.runTarget", "Run Target", Icon::Play, None),
            Command::new(
                "projectOutline.debugTarget",
                "Debug Target",
                Icon::DebugAlt,
                None,
            ),
            Command::new(
                "projectOutline.benchTarget",
                "Benchmark Target",
                Icon::Dashboard,
                None,
            ),
            Command::new(
                "projectOutline.setWorkspaceMemberFilter",
                "Filter Workspace Members",
                Icon::Filter,
                None,
            ),
            Command::new(
                "projectOutline.editWorkspaceMemberFilter",
                "Edit Member Filter",
                Icon::Edit,
                None,
            ),
            Command::new(
                "projectOutline.clearWorkspaceMemberFilter",
                "Clear Member Filter",
                Icon::ClearAll,
                None,
            ),
            Command::new(
                "projectOutline.showTargetTypeFilter",
                "Filter Target Types",
                Icon::SymbolClass,
                None,
            ),
            Command::new(
                "projectOutline.clearTargetTypeFilter",
                "Clear Type Filter",
                Icon::ClearAll,
                None,
            ),
            Command::new(
                "projectOutline.clearAllFilters",
                "Clear All Filters",
                Icon::ClearAll,
                None,
            ),
            Command::new(
                "projectOutline.toggleWorkspaceMemberGrouping",
                "Toggle Workspace Member Grouping",
                Icon::GroupByRefType,
                None,
            ),
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
                id: ViewId::CargoToolsExplorer,
                name: "Cargo Tools".to_string(),
                icon: "$(package)".to_string(),
                when: Some("cargoTools:workspaceHasCargo".to_string()),
            }],
            cargo_tools: vec![
                View {
                    id: ViewId::CargoToolsProjectStatus,
                    name: "Project Status".to_string(),
                    icon: "$(package)".to_string(),
                    when: Some("cargoTools:workspaceHasCargo".to_string()),
                },
                View {
                    id: ViewId::CargoToolsProjectOutline,
                    name: "Project Outline".to_string(),
                    icon: "$(package)".to_string(),
                    when: Some("cargoTools:workspaceHasCargo".to_string()),
                },
                View {
                    id: ViewId::CargoToolsMakefile,
                    name: "Makefile".to_string(),
                    icon: "$(tools)".to_string(),
                    when: Some(
                        "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile"
                            .to_string(),
                    ),
                },
                View {
                    id: ViewId::CargoToolsPinnedMakefileTasks,
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
        use MenuGroupType::*;

        Menus {
            view_title: vec![
                // Project Status view
                MenuItem::for_view_title(
                    "cargo-tools.refresh",
                    "cargoToolsProjectStatus",
                    MenuGroup::new(Navigation),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.clean",
                    "cargoToolsProjectStatus",
                    MenuGroup::new(Navigation),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.buildDocs",
                    "cargoToolsProjectStatus",
                    MenuGroup::new(Navigation),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.installPlatformTarget",
                    "cargoToolsProjectStatus",
                    MenuGroup::new(Navigation),
                ),
                // Project Outline view
                MenuItem::for_view_title(
                    "cargo-tools.projectOutline.setWorkspaceMemberFilter",
                    "cargoToolsProjectOutline",
                    MenuGroup::with_position(Navigation, 1),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.projectOutline.showTargetTypeFilter",
                    "cargoToolsProjectOutline",
                    MenuGroup::with_position(Navigation, 2),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.projectOutline.clearAllFilters",
                    "cargoToolsProjectOutline",
                    MenuGroup::with_position(Navigation, 3),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping",
                    "cargoToolsProjectOutline",
                    MenuGroup::with_position(Navigation, 4),
                ),
                // Makefile view
                MenuItem::for_view_title(
                    "cargo-tools.makefile.setTaskFilter",
                    "cargoToolsMakefile",
                    MenuGroup::with_position(Navigation, 1),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.makefile.showCategoryFilter",
                    "cargoToolsMakefile",
                    MenuGroup::with_position(Navigation, 2),
                ),
                MenuItem::for_view_title(
                    "cargo-tools.makefile.clearTaskFilter",
                    "cargoToolsMakefile",
                    MenuGroup::with_position(Navigation, 3),
                ),
                // Pinned Makefile Tasks view
                MenuItem::for_view_title(
                    "cargo-tools.pinnedMakefileTasks.add",
                    "cargoToolsPinnedMakefileTasks",
                    MenuGroup::with_position(Navigation, 1),
                ),
            ],
            view_item_context: vec![
                // Package selection
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.selectPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember.*canBeSelectedPackage",
                    MenuGroup::with_position(Selection, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.unselectPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember.*isSelectedPackage",
                    MenuGroup::with_position(Selection, 2),
                ),
                // Build target selection
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.setBuildTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*canBeSelectedBuildTarget",
                    MenuGroup::with_position(Build, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.unsetBuildTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*isSelectedBuildTarget",
                    MenuGroup::with_position(Build, 2),
                ),
                // Run target selection
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.setRunTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*canBeSelectedRunTarget",
                    MenuGroup::with_position(Run, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.unsetRunTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*isSelectedRunTarget",
                    MenuGroup::with_position(Run, 2),
                ),
                // Benchmark target selection
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.setBenchmarkTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*canBeSelectedBenchmarkTarget",
                    MenuGroup::with_position(Benchmark, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.unsetBenchmarkTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*isSelectedBenchmarkTarget",
                    MenuGroup::with_position(Benchmark, 2),
                ),
                // Package actions (context menu)
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.buildPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember",
                    MenuGroup::with_position(Actions, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.testPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember",
                    MenuGroup::with_position(Actions, 2),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.cleanPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember",
                    MenuGroup::with_position(Actions, 3),
                ),
                // Workspace actions (context menu)
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.buildWorkspace",
                    "cargoToolsProjectOutline",
                    "project",
                    MenuGroup::with_position(Actions, 1),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.testWorkspace",
                    "cargoToolsProjectOutline",
                    "project",
                    MenuGroup::with_position(Actions, 2),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.cleanWorkspace",
                    "cargoToolsProjectOutline",
                    "project",
                    MenuGroup::with_position(Actions, 3),
                ),
                // Target actions (context menu)
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.buildTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsBuild",
                    MenuGroup::with_position(Actions, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.runTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsRun",
                    MenuGroup::with_position(Actions, 2),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.debugTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsDebug",
                    MenuGroup::with_position(Actions, 3),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.benchTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsBench",
                    MenuGroup::with_position(Actions, 4),
                ),
                // Package inline actions
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.buildPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember",
                    MenuGroup::with_position(Inline, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.testPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember",
                    MenuGroup::with_position(Inline, 2),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.cleanPackage",
                    "cargoToolsProjectOutline",
                    "workspaceMember",
                    MenuGroup::with_position(Inline, 3),
                ),
                // Workspace inline actions
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.buildWorkspace",
                    "cargoToolsProjectOutline",
                    "project",
                    MenuGroup::with_position(Inline, 1),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.testWorkspace",
                    "cargoToolsProjectOutline",
                    "project",
                    MenuGroup::with_position(Inline, 2),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.cleanWorkspace",
                    "cargoToolsProjectOutline",
                    "project",
                    MenuGroup::with_position(Inline, 3),
                ),
                // Target inline actions
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.buildTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsBuild",
                    MenuGroup::with_position(Inline, 1),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.runTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsRun",
                    MenuGroup::with_position(Inline, 2),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.debugTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsDebug",
                    MenuGroup::with_position(Inline, 3),
                ),
                MenuItem::for_view_item_context_regex(
                    "cargo-tools.projectOutline.benchTarget",
                    "cargoToolsProjectOutline",
                    "cargoTarget.*supportsBench",
                    MenuGroup::with_position(Inline, 4),
                ),
                // Project Status inline actions
                MenuItem::for_view_item_context(
                    "cargo-tools.projectStatus.build",
                    "cargoToolsProjectStatus",
                    "buildTargetSelection",
                    MenuGroup::new(Inline),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectStatus.run",
                    "cargoToolsProjectStatus",
                    "runTargetSelection",
                    MenuGroup::with_position(Inline, 1),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectStatus.debug",
                    "cargoToolsProjectStatus",
                    "runTargetSelection",
                    MenuGroup::with_position(Inline, 2),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectStatus.test",
                    "cargoToolsProjectStatus",
                    "packageSelection",
                    MenuGroup::new(Inline),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectStatus.bench",
                    "cargoToolsProjectStatus",
                    "benchmarkTargetSelection",
                    MenuGroup::new(Inline),
                ),
                // Filter inline/modify actions
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.setWorkspaceMemberFilter",
                    "cargoToolsProjectOutline",
                    "memberFilter",
                    MenuGroup::new(Inline),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.clearWorkspaceMemberFilter",
                    "cargoToolsProjectOutline",
                    "memberFilter",
                    MenuGroup::with_position(Modify, 1),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.showTargetTypeFilter",
                    "cargoToolsProjectOutline",
                    "typeFilter",
                    MenuGroup::new(Inline),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.projectOutline.clearTargetTypeFilter",
                    "cargoToolsProjectOutline",
                    "typeFilter",
                    MenuGroup::with_position(Modify, 1),
                ),
                MenuItem {
                    command: "cargo-tools.projectOutline.clearAllFilters".to_string(),
                    when: Some(
                        "view == cargoToolsProjectOutline && viewItem =~ /memberFilter|typeFilter/"
                            .to_string(),
                    ),
                    group: Some(MenuGroup::with_position(Modify, 2)),
                },
                // Makefile task actions
                MenuItem::for_view_item_context(
                    "cargo-tools.makefile.runTask",
                    "cargoToolsMakefile",
                    "makefileTask",
                    MenuGroup::with_position(Inline, 1),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.makefile.pinTask",
                    "cargoToolsMakefile",
                    "makefileTask",
                    MenuGroup::with_position(Context, 1),
                ),
                // Pinned task actions
                MenuItem::for_view_item_context(
                    "cargo-tools.pinnedMakefileTasks.execute",
                    "cargoToolsPinnedMakefileTasks",
                    "pinned-task",
                    MenuGroup::with_position(Inline, 1),
                ),
                MenuItem::for_view_item_context(
                    "cargo-tools.pinnedMakefileTasks.remove",
                    "cargoToolsPinnedMakefileTasks",
                    "pinned-task",
                    MenuGroup::with_position(Context, 1),
                ),
            ],
            command_palette: vec![
                MenuItem::hide_from_palette("cargo-tools.projectOutline.selectPackage"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.unselectPackage"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.setBuildTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.unsetBuildTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.setRunTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.unsetRunTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.setBenchmarkTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.unsetBenchmarkTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.buildPackage"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.testPackage"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.buildTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.runTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.benchTarget"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.setWorkspaceMemberFilter"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.editWorkspaceMemberFilter"),
                MenuItem::hide_from_palette(
                    "cargo-tools.projectOutline.clearWorkspaceMemberFilter",
                ),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.showTargetTypeFilter"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.clearTargetTypeFilter"),
                MenuItem::hide_from_palette("cargo-tools.projectOutline.clearAllFilters"),
                MenuItem::hide_from_palette(
                    "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping",
                ),
                MenuItem::hide_from_palette("cargo-tools.makefile.runTask"),
                MenuItem::hide_from_palette("cargo-tools.makefile.pinTask"),
                MenuItem::hide_from_palette("cargo-tools.pinnedMakefileTasks.add"),
                MenuItem::hide_from_palette("cargo-tools.pinnedMakefileTasks.remove"),
                MenuItem::hide_from_palette("cargo-tools.pinnedMakefileTasks.execute"),
            ],
        }
    }

    pub fn extension_configuration() -> Configuration {
        let mut properties = HashMap::new();
        properties.insert(
            "cargoTools.cargoCommand".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::String,
                default: Some(serde_json::json!("cargo")),
                description: Some("Command to invoke instead of 'cargo'. This can be a custom wrapper or alternative cargo implementation. If the value contains whitespace, it will be split where the first part is the command and the remaining parts are treated as additional arguments.".to_string()),
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.useRustAnalyzerEnvAndArgs".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Boolean,
                default: Some(serde_json::json!(false)),
                description: Some("Use rust-analyzer settings for cargo command and environment. When enabled, reads rust-analyzer.cargo.extraArgs, rust-analyzer.cargo.extraEnv, rust-analyzer.runnables.extraArgs, and rust-analyzer.runnables.extraTestBinaryArgs to configure this extension.".to_string()),
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.updateRustAnalyzerTarget".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Boolean,
                default: Some(serde_json::json!(false)),
                description: Some("Automatically update rust-analyzer.cargo.target setting when Platform Selection changes. When enabled, changing the platform target will also set rust-analyzer's cargo target configuration.".to_string()),
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.extraEnv".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Object,
                default: Some(serde_json::json!({})),
                description: Some(
                    "Additional environment variables to set when running any cargo command"
                        .to_string(),
                ),
                items: None,
                additional_properties: Some(Box::new(ConfigurationProperty {
                    type_: ConfigPropertyType::String,
                    default: None,
                    description: None,
                    items: None,
                    additional_properties: None,
                })),
            },
        );
        properties.insert(
            "cargoTools.buildArgs".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Array,
                default: Some(serde_json::json!([])),
                description: Some("Additional arguments to pass to cargo build".to_string()),
                items: Some(Box::new(ConfigurationProperty {
                    type_: ConfigPropertyType::String,
                    default: None,
                    description: None,
                    items: None,
                    additional_properties: None,
                })),
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.run.extraArgs".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Array,
                default: Some(serde_json::json!([])),
                description: Some("Additional arguments to append to each invocation of running or debugging a target".to_string()),
                items: Some(Box::new(ConfigurationProperty {
                    type_: ConfigPropertyType::String,
                    default: None,
                    description: None,
                    items: None,
                    additional_properties: None,
                })),
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.run.extraEnv".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Object,
                default: Some(serde_json::json!({})),
                description: Some("Additional environment variables to set when running or debugging a target, merged with extraEnv".to_string()),
                items: None,
                additional_properties: Some(Box::new(ConfigurationProperty {
                    type_: ConfigPropertyType::String,
                    default: None,
                    description: None,
                    items: None,
                    additional_properties: None,
                })),
            },
        );
        properties.insert(
            "cargoTools.test.extraArgs".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Array,
                default: Some(serde_json::json!([])),
                description: Some("Additional arguments to append to each invocation of running tests or benchmarks".to_string()),
                items: Some(Box::new(ConfigurationProperty {
                    type_: ConfigPropertyType::String,
                    default: None,
                    description: None,
                    items: None,
                    additional_properties: None,
                })),
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.test.extraEnv".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::Object,
                default: Some(serde_json::json!({})),
                description: Some("Additional environment variables to set when running tests or benchmarks, merged with extraEnv".to_string()),
                items: None,
                additional_properties: Some(Box::new(ConfigurationProperty {
                    type_: ConfigPropertyType::String,
                    default: None,
                    description: None,
                    items: None,
                    additional_properties: None,
                })),
            },
        );
        properties.insert(
            "cargoTools.runCommandOverride".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::String,
                default: Some(serde_json::json!("")),
                description: Some("Override command for 'cargo run'. If empty, 'cargo run' will be used. Use this to customize the run command (e.g., 'cargo watch -x run' or custom scripts).".to_string()),
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "cargoTools.testCommandOverride".to_string(),
            ConfigurationProperty {
                type_: ConfigPropertyType::String,
                default: Some(serde_json::json!("")),
                description: Some("Override command for 'cargo test'. If empty, 'cargo test' will be used. Use this to customize the test command (e.g., 'cargo nextest run' or custom scripts).".to_string()),
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
        vec![TaskDefinition::cargo(), TaskDefinition::cargo_make()]
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
            Keybinding {
                command: "cargo-tools.pinnedMakefileTasks.execute1".to_string(),
                key: "ctrl+alt+1".to_string(),
                when: Some(
                    "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string(),
                ),
            },
            Keybinding {
                command: "cargo-tools.pinnedMakefileTasks.execute2".to_string(),
                key: "ctrl+alt+2".to_string(),
                when: Some(
                    "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string(),
                ),
            },
            Keybinding {
                command: "cargo-tools.pinnedMakefileTasks.execute3".to_string(),
                key: "ctrl+alt+3".to_string(),
                when: Some(
                    "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string(),
                ),
            },
            Keybinding {
                command: "cargo-tools.pinnedMakefileTasks.execute4".to_string(),
                key: "ctrl+alt+4".to_string(),
                when: Some(
                    "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string(),
                ),
            },
            Keybinding {
                command: "cargo-tools.pinnedMakefileTasks.execute5".to_string(),
                key: "ctrl+alt+5".to_string(),
                when: Some(
                    "cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string(),
                ),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn view_serializes_with_correct_field_names() {
        let view = View {
            id: ViewId::CargoToolsExplorer,
            name: "Test View".to_string(),
            icon: "$(package)".to_string(),
            when: Some("test:condition".to_string()),
        };

        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["id"], "cargoToolsExplorer");
        assert_eq!(json["name"], "Test View");
        assert_eq!(json["when"], "test:condition");
        assert_eq!(json["icon"], "$(package)");
    }

    #[test]
    fn task_definition_with_properties() {
        let mut properties = HashMap::new();
        properties.insert(
            "command".to_string(),
            TaskProperty::string(Some("The cargo command".to_string())),
        );

        let task = TaskDefinition {
            type_: TaskType::Cargo,
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
            group: Some(MenuGroup::new(MenuGroupType::Navigation)),
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

        let package_contributes: Contributes =
            serde_json::from_value(package["contributes"].clone())
                .expect("Failed to deserialize contributes from package.json");

        assert_eq!(
            &package_contributes,
            &*data::CONTRIBUTES,
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
                prop.description.as_ref().is_some_and(|d| !d.is_empty()),
                "Property {} should have a non-empty description",
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
                command.category, "Cargo Tools",
                "Command {} should have category 'Cargo Tools'",
                command.command
            );
            // Icon is now an enum, so it always has a value - no need to check
        }
    }

    #[test]
    fn all_views_have_consistent_naming() {
        let views = data::all_views();

        for view in &views.explorer {
            let id = serde_json::to_string(&view.id).unwrap();
            assert!(
                id.starts_with("\"cargoTools"),
                "Explorer view {id:?} should start with 'cargoTools'"
            );
            assert!(!view.name.is_empty(), "View {id:?} should have a name");
        }

        for view in &views.cargo_tools {
            let id = serde_json::to_string(&view.id).unwrap();
            assert!(
                id.starts_with("\"cargoTools"),
                "CargoTools view {:?} should start with 'cargoTools'",
                view.id
            );
            assert!(!view.name.is_empty(), "View {id:?} should have a name");
        }
    }

    #[test]
    fn task_definitions_have_required_properties() {
        let task_defs = data::all_task_definitions();

        for task_def in &task_defs {
            // TaskType is now an enum, so it always has a value
            assert!(
                !task_def.required.is_empty(),
                "Task definition {:?} should have required properties",
                task_def.type_
            );

            for required_prop in &task_def.required {
                assert!(
                    task_def.properties.contains_key(required_prop),
                    "Task definition {:?} should define required property {}",
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
                    "Property {} in task {:?} has invalid type: {}",
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
