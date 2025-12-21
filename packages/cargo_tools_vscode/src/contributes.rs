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
            Command::new("testRust", "Test Rust bindgen", "$(gear)", None),
            Command::new("selectProfile", "Select Build Profile", "$(gear)", None),
            Command::new("selectPackage", "Select Package", "$(package)", None),
            Command::new("selectBuildTarget", "Select Build Target", "$(tools)", None),
            Command::new("selectRunTarget", "Select Run Target", "$(play)", None),
            Command::new("selectBenchmarkTarget", "Select Benchmark Target", "$(dashboard)", None),
            Command::new("selectPlatformTarget", "Select Platform Target", "$(desktop-download)", None),
            Command::new("installPlatformTarget", "Install Platform Target", "$(cloud-download)", None),
            Command::new("setRustAnalyzerCheckTargets", "Set rust-analyzer check targets", "$(checklist)", None),
            Command::new("buildDocs", "Build Documentation", "$(book)", None),
            Command::new("selectFeatures", "Select Features", "$(extensions)", None),
            Command::new("refresh", "Refresh", "$(refresh)", None),
            Command::new("clean", "Clean Build Artifacts", "$(trash)", None),
            Command::new(
                "makefile.runTask",
                "Run Makefile Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.selectAndRunTask",
                "Run Makefile Task...",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.setTaskFilter",
                "Filter Tasks",
                "$(filter)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.editTaskFilter",
                "Edit Task Filter",
                "$(edit)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.clearTaskFilter",
                "Clear Task Filter",
                "$(clear-all)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.showCategoryFilter",
                "Filter Categories",
                "$(symbol-class)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.clearCategoryFilter",
                "Clear Category Filter",
                "$(clear-all)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.add",
                "Add Task",
                "$(add)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.remove",
                "Remove Task",
                "$(remove)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute",
                "Execute Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "makefile.pinTask",
                "Pin Task",
                "$(pin)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute1",
                "Execute 1st Pinned Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute2",
                "Execute 2nd Pinned Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute3",
                "Execute 3rd Pinned Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute4",
                "Execute 4th Pinned Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new(
                "pinnedMakefileTasks.execute5",
                "Execute 5th Pinned Task",
                "$(play)",
                Some("cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile".to_string()),
            ),
            Command::new("projectStatus.build", "Build", "$(tools)", None),
            Command::new("projectStatus.run", "Run", "$(play)", None),
            Command::new("projectStatus.debug", "Debug", "$(debug-alt)", None),
            Command::new("projectStatus.test", "Test", "$(beaker)", None),
            Command::new("projectStatus.bench", "Benchmark", "$(dashboard)", None),
            Command::new("projectOutline.selectPackage", "Select Package", "$(check)", None),
            Command::new("projectOutline.unselectPackage", "Unselect Package", "$(close)", None),
            Command::new("projectOutline.setBuildTarget", "Set as Build Target", "$(tools)", None),
            Command::new("projectOutline.unsetBuildTarget", "Unset Build Target", "$(close)", None),
            Command::new("projectOutline.setRunTarget", "Set as Run Target", "$(play)", None),
            Command::new("projectOutline.unsetRunTarget", "Unset Run Target", "$(close)", None),
            Command::new("projectOutline.setBenchmarkTarget", "Set as Benchmark Target", "$(dashboard)", None),
            Command::new("projectOutline.unsetBenchmarkTarget", "Unset Benchmark Target", "$(close)", None),
            Command::new("projectOutline.buildPackage", "Build Package", "$(tools)", None),
            Command::new("projectOutline.testPackage", "Test Package", "$(beaker)", None),
            Command::new("projectOutline.cleanPackage", "Clean Package", "$(trash)", None),
            Command::new("projectOutline.buildWorkspace", "Build Workspace", "$(tools)", None),
            Command::new("projectOutline.testWorkspace", "Test Workspace", "$(beaker)", None),
            Command::new("projectOutline.cleanWorkspace", "Clean Workspace", "$(trash)", None),
            Command::new("projectOutline.buildTarget", "Build Target", "$(tools)", None),
            Command::new("projectOutline.runTarget", "Run Target", "$(play)", None),
            Command::new("projectOutline.debugTarget", "Debug Target", "$(debug-alt)", None),
            Command::new("projectOutline.benchTarget", "Benchmark Target", "$(dashboard)", None),
            Command::new("projectOutline.setWorkspaceMemberFilter", "Filter Workspace Members", "$(filter)", None),
            Command::new("projectOutline.editWorkspaceMemberFilter", "Edit Member Filter", "$(edit)", None),
            Command::new("projectOutline.clearWorkspaceMemberFilter", "Clear Member Filter", "$(clear-all)", None),
            Command::new("projectOutline.showTargetTypeFilter", "Filter Target Types", "$(symbol-class)", None),
            Command::new("projectOutline.clearTargetTypeFilter", "Clear Type Filter", "$(clear-all)", None),
            Command::new("projectOutline.clearAllFilters", "Clear All Filters", "$(clear-all)", None),
            Command::new("projectOutline.toggleWorkspaceMemberGrouping", "Toggle Workspace Member Grouping", "$(group-by-ref-type)", None),
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
                MenuItem {
                    command: "cargo-tools.buildDocs".to_string(),
                    when: Some("view == cargoToolsProjectStatus".to_string()),
                    group: Some("navigation".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.installPlatformTarget".to_string(),
                    when: Some("view == cargoToolsProjectStatus".to_string()),
                    group: Some("navigation".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setWorkspaceMemberFilter".to_string(),
                    when: Some("view == cargoToolsProjectOutline".to_string()),
                    group: Some("navigation@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.showTargetTypeFilter".to_string(),
                    when: Some("view == cargoToolsProjectOutline".to_string()),
                    group: Some("navigation@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearAllFilters".to_string(),
                    when: Some("view == cargoToolsProjectOutline".to_string()),
                    group: Some("navigation@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping".to_string(),
                    when: Some("view == cargoToolsProjectOutline".to_string()),
                    group: Some("navigation@4".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.makefile.setTaskFilter".to_string(),
                    when: Some("view == cargoToolsMakefile".to_string()),
                    group: Some("navigation@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.makefile.showCategoryFilter".to_string(),
                    when: Some("view == cargoToolsMakefile".to_string()),
                    group: Some("navigation@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.makefile.clearTaskFilter".to_string(),
                    when: Some("view == cargoToolsMakefile".to_string()),
                    group: Some("navigation@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.pinnedMakefileTasks.add".to_string(),
                    when: Some("view == cargoToolsPinnedMakefileTasks".to_string()),
                    group: Some("navigation@1".to_string()),
                },
            ],
            view_item_context: vec![
                MenuItem {
                    command: "cargo-tools.projectOutline.selectPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember.*canBeSelectedPackage/".to_string()),
                    group: Some("selection@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unselectPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember.*isSelectedPackage/".to_string()),
                    group: Some("selection@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setBuildTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*canBeSelectedBuildTarget/".to_string()),
                    group: Some("build@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unsetBuildTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*isSelectedBuildTarget/".to_string()),
                    group: Some("build@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setRunTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*canBeSelectedRunTarget/".to_string()),
                    group: Some("run@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unsetRunTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*isSelectedRunTarget/".to_string()),
                    group: Some("run@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setBenchmarkTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*canBeSelectedBenchmarkTarget/".to_string()),
                    group: Some("benchmark@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unsetBenchmarkTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*isSelectedBenchmarkTarget/".to_string()),
                    group: Some("benchmark@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/".to_string()),
                    group: Some("actions@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.testPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/".to_string()),
                    group: Some("actions@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.cleanPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/".to_string()),
                    group: Some("actions@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildWorkspace".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == project".to_string()),
                    group: Some("actions@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.testWorkspace".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == project".to_string()),
                    group: Some("actions@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.cleanWorkspace".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == project".to_string()),
                    group: Some("actions@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsBuild/".to_string()),
                    group: Some("actions@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.runTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsRun/".to_string()),
                    group: Some("actions@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.debugTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsDebug/".to_string()),
                    group: Some("actions@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.benchTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsBench/".to_string()),
                    group: Some("actions@4".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/".to_string()),
                    group: Some("inline@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.testPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/".to_string()),
                    group: Some("inline@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.cleanPackage".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/".to_string()),
                    group: Some("inline@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildWorkspace".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == project".to_string()),
                    group: Some("inline@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.testWorkspace".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == project".to_string()),
                    group: Some("inline@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.cleanWorkspace".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == project".to_string()),
                    group: Some("inline@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsBuild/".to_string()),
                    group: Some("inline@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.runTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsRun/".to_string()),
                    group: Some("inline@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.debugTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsDebug/".to_string()),
                    group: Some("inline@3".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.benchTarget".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /cargoTarget.*supportsBench/".to_string()),
                    group: Some("inline@4".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectStatus.build".to_string(),
                    when: Some("view == cargoToolsProjectStatus && viewItem == buildTargetSelection".to_string()),
                    group: Some("inline".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectStatus.run".to_string(),
                    when: Some("view == cargoToolsProjectStatus && viewItem == runTargetSelection".to_string()),
                    group: Some("inline@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectStatus.debug".to_string(),
                    when: Some("view == cargoToolsProjectStatus && viewItem == runTargetSelection".to_string()),
                    group: Some("inline@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectStatus.test".to_string(),
                    when: Some("view == cargoToolsProjectStatus && viewItem == packageSelection".to_string()),
                    group: Some("inline".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectStatus.bench".to_string(),
                    when: Some("view == cargoToolsProjectStatus && viewItem == benchmarkTargetSelection".to_string()),
                    group: Some("inline".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setWorkspaceMemberFilter".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == memberFilter".to_string()),
                    group: Some("inline".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearWorkspaceMemberFilter".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == memberFilter".to_string()),
                    group: Some("modify@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.showTargetTypeFilter".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == typeFilter".to_string()),
                    group: Some("inline".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearTargetTypeFilter".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem == typeFilter".to_string()),
                    group: Some("modify@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearAllFilters".to_string(),
                    when: Some("view == cargoToolsProjectOutline && viewItem =~ /memberFilter|typeFilter/".to_string()),
                    group: Some("modify@2".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.makefile.runTask".to_string(),
                    when: Some("view == cargoToolsMakefile && viewItem == makefileTask".to_string()),
                    group: Some("inline@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.makefile.pinTask".to_string(),
                    when: Some("view == cargoToolsMakefile && viewItem == makefileTask".to_string()),
                    group: Some("context@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.pinnedMakefileTasks.execute".to_string(),
                    when: Some("view == cargoToolsPinnedMakefileTasks && viewItem == pinned-task".to_string()),
                    group: Some("inline@1".to_string()),
                },
                MenuItem {
                    command: "cargo-tools.pinnedMakefileTasks.remove".to_string(),
                    when: Some("view == cargoToolsPinnedMakefileTasks && viewItem == pinned-task".to_string()),
                    group: Some("context@1".to_string()),
                },
            ],
            command_palette: vec![
                MenuItem {
                    command: "cargo-tools.projectOutline.selectPackage".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unselectPackage".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setBuildTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unsetBuildTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setRunTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unsetRunTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setBenchmarkTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.unsetBenchmarkTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildPackage".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.testPackage".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.buildTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.runTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.benchTarget".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.setWorkspaceMemberFilter".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.editWorkspaceMemberFilter".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearWorkspaceMemberFilter".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.showTargetTypeFilter".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearTargetTypeFilter".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.clearAllFilters".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.makefile.runTask".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.makefile.pinTask".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.pinnedMakefileTasks.add".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.pinnedMakefileTasks.remove".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
                MenuItem {
                    command: "cargo-tools.pinnedMakefileTasks.execute".to_string(),
                    when: Some("never".to_string()),
                    group: None,
                },
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
        cargo_props.insert(
            "features".to_string(),
            TaskProperty {
                type_: "array".to_string(),
                description: Some("Features to enable".to_string()),
                items: Some(Box::new(TaskProperty {
                    type_: "string".to_string(),
                    description: None,
                    items: None,
                })),
            },
        );
        cargo_props.insert(
            "allFeatures".to_string(),
            TaskProperty {
                type_: "boolean".to_string(),
                description: Some("Enable all features".to_string()),
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
                command.category,
                "Cargo Tools",
                "Command {} should have category 'Cargo Tools'",
                command.command
            );
            assert!(
                !command.icon.is_empty(),
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
