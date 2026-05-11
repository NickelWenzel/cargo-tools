use std::{collections::HashMap, iter};

use cargo_tools::cargo::{
    Config, Features,
    command::{BenchTarget, BuildSubTarget, BuildTarget, RunSubTarget, RunTarget},
    metadata::{self, Package, Target, TargetType},
};
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    extension::cargo::Grouping,
    icon::{
        BENCH_ACTION, BENCH_TARGET, BIN_TARGET, BUILD_ACTION, EXAMPLE_TARGET, FEATURES_CONFIG,
        Icon, LIB_TARGET, PACKAGE, PLATFORM_CONFIG, PROFILE_CONFIG, PROJECT, RUN_ACTION,
        SELECTED_STATE, TARGET_CONFIG, UNSELECTED_STATE,
    },
    vs_code_api::{CargoNode, CargoOutlineNode, log_error},
};

const PACKAGE_CTX: &str = "packageSelection";
const BUILD_CTX: &str = "buildTargetSelection";
const RUN_CTX: &str = "runTargetSelection";
const BENCH_CTX: &str = "benchmarkTargetSelection";

#[derive(Debug, Clone)]
pub struct ConfigUiRequest {
    pub tx: Sender<Vec<NodeData>>,
    pub node_type: Option<NodeType>,
}

#[derive(Debug, Clone, Copy)]
#[wasm_bindgen]
pub struct NodeType(NodeTypeInner);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum NodeTypeInner {
    Platform,
    BuildConfig,
    Package,
    Target,
    Features,
    BuildTarget,
    RunTarget,
    BenchTarget,
    Selection,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    label: String,
    icon: Icon,
    collapsible_state: CollapsibleState,
    node_type: NodeType,
    context_value: Option<String>,
    tooltip: Option<String>,
    command: Option<String>,
    command_arg: Option<String>,
}

impl NodeData {
    fn into_node(self) -> CargoNode {
        let Self {
            label,
            icon,
            collapsible_state,
            node_type,
            context_value,
            tooltip,
            command,
            command_arg,
        } = self;

        CargoNode::new(
            label,
            icon,
            collapsible_state as u32,
            node_type,
            context_value,
            None,
            tooltip,
            command,
            command_arg,
        )
    }

    fn node(label: String, icon: Icon, handler: NodeType, context_value: Option<String>) -> Self {
        Self {
            label,
            icon,
            collapsible_state: CollapsibleState::Expanded,
            node_type: handler,
            context_value,
            tooltip: None,
            command: None,
            command_arg: None,
        }
    }

    fn leaf(
        label: String,
        icon: Icon,
        handler: NodeType,
        tooltip: String,
        command: String,
        command_arg: Option<String>,
    ) -> Self {
        Self {
            label,
            icon,
            collapsible_state: CollapsibleState::None,
            node_type: handler,
            context_value: None,
            tooltip: Some(tooltip),
            command: Some(command),
            command_arg,
        }
    }

    pub fn roots() -> Vec<Self> {
        let platform = Self::node(
            "Target platform".to_string(),
            PLATFORM_CONFIG,
            NodeType::platform(),
            None,
        );
        let build_config = Self::node(
            "Build Configuration".to_string(),
            PROFILE_CONFIG,
            NodeType::build_config(),
            None,
        );
        let package = Self::node(
            "Packages".to_string(),
            PACKAGE,
            NodeType::package(),
            Some(PACKAGE_CTX.to_string()),
        );
        let target = Self::node(
            "Targets".to_string(),
            TARGET_CONFIG,
            NodeType::target(),
            None,
        );
        let feature = Self::node(
            "Features".to_string(),
            FEATURES_CONFIG,
            NodeType::features(),
            None,
        );

        vec![platform, build_config, package, target, feature]
    }
}

impl NodeType {
    pub fn children(&self, config: &Config, available_features: &[String]) -> Vec<NodeData> {
        let default = "Default".to_string();
        match &self.0 {
            NodeTypeInner::Platform => vec![NodeData::leaf(
                config.platform_target.clone().unwrap_or(default),
                PLATFORM_CONFIG,
                NodeType::selection(),
                "Click to select target platform".to_string(),
                "cargo-tools.selectPlatformTarget".to_string(),
                None,
            )],
            NodeTypeInner::BuildConfig => vec![NodeData::leaf(
                config
                    .profile
                    .get_name()
                    .unwrap_or("Default (dev)")
                    .to_string(),
                PROFILE_CONFIG,
                NodeType::selection(),
                "Click to select build profile".to_string(),
                "cargo-tools.selectProfile".to_string(),
                None,
            )],
            NodeTypeInner::Package => vec![NodeData::leaf(
                config
                    .selected_package
                    .clone()
                    .unwrap_or("No selection".to_string()),
                PACKAGE,
                NodeType::selection(),
                "Click to select package".to_string(),
                "cargo-tools.selectPackage".to_string(),
                None,
            )],
            NodeTypeInner::Target => {
                let build = NodeData::node(
                    "Build target".to_string(),
                    BUILD_ACTION,
                    NodeType::build_target(),
                    Some(BUILD_CTX.to_string()),
                );

                let run = NodeData::node(
                    "Run target".to_string(),
                    RUN_ACTION,
                    NodeType::run_target(),
                    Some(RUN_CTX.to_string()),
                );

                let bench = NodeData::node(
                    "Benchmark target".to_string(),
                    BENCH_ACTION,
                    NodeType::bench_target(),
                    Some(BENCH_CTX.to_string()),
                );

                vec![build, run, bench]
            }
            NodeTypeInner::Features => feature_leaves(config, available_features),
            NodeTypeInner::BuildTarget => {
                let build_target = config
                    .package_selection()
                    .and_then(|p| p.build_target.clone());
                vec![NodeData::leaf(
                    build_target
                        .as_ref()
                        .map(|t| t.name().to_string())
                        .clone()
                        .unwrap_or(default),
                    Icon::build_target(&build_target),
                    NodeType::selection(),
                    "Click to select build target".to_string(),
                    "cargo-tools.selectBuildTarget".to_string(),
                    None,
                )]
            }
            NodeTypeInner::RunTarget => {
                let run_target = config
                    .package_selection()
                    .and_then(|p| p.run_target.clone());
                vec![NodeData::leaf(
                    run_target
                        .as_ref()
                        .map(|t| t.name().to_string())
                        .clone()
                        .unwrap_or(default),
                    Icon::run_target(&run_target),
                    NodeType::selection(),
                    "Click to select run target".to_string(),
                    "cargo-tools.selectRunTarget".to_string(),
                    None,
                )]
            }
            NodeTypeInner::BenchTarget => vec![NodeData::leaf(
                config
                    .package_selection()
                    .and_then(|p| p.benchmark_target.clone())
                    .unwrap_or(default),
                BENCH_TARGET,
                NodeType::selection(),
                "Click to select benchmark target".to_string(),
                "cargo-tools.selectBenchmarkTarget".to_string(),
                None,
            )],
            NodeTypeInner::Selection => Vec::new(),
        }
    }
}

impl NodeType {
    fn platform() -> Self {
        Self(NodeTypeInner::Platform)
    }
    fn build_config() -> Self {
        Self(NodeTypeInner::BuildConfig)
    }
    fn package() -> Self {
        Self(NodeTypeInner::Package)
    }
    fn target() -> Self {
        Self(NodeTypeInner::Target)
    }
    fn features() -> Self {
        Self(NodeTypeInner::Features)
    }
    fn build_target() -> Self {
        Self(NodeTypeInner::BuildTarget)
    }
    fn run_target() -> Self {
        Self(NodeTypeInner::RunTarget)
    }
    fn bench_target() -> Self {
        Self(NodeTypeInner::BenchTarget)
    }
    fn selection() -> Self {
        Self(NodeTypeInner::Selection)
    }
}

#[wasm_bindgen]
pub struct CargoConfigurationTreeProviderHandler {
    tx: Sender<ConfigUiRequest>,
}

#[wasm_bindgen]
impl CargoConfigurationTreeProviderHandler {
    pub async fn children(&self, node_type: Option<NodeType>) -> Vec<CargoNode> {
        let (tx, mut rx) = channel(1);

        let request = ConfigUiRequest { tx, node_type };

        if let Err(e) = self.tx.clone().send(request).await {
            log_error(&format!("Failed to send UiConfigUpdate: {e}"));
        }

        rx.next()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(NodeData::into_node)
            .collect()
    }
}

impl CargoConfigurationTreeProviderHandler {
    pub fn new(tx: Sender<ConfigUiRequest>) -> Self {
        Self { tx }
    }
}

fn feature_leaves(config: &Config, available_features: &[String]) -> Vec<NodeData> {
    available_features
        .iter()
        .map(|feat| {
            let selected = match config.selected_features() {
                Features::All => feat == "All features",
                Features::Some(selected) => selected.contains(feat),
            };
            let icon = if selected {
                SELECTED_STATE
            } else {
                UNSELECTED_STATE
            };

            NodeData::leaf(
                feat.to_string(),
                icon,
                NodeType::selection(),
                "Toggle feature".to_string(),
                "cargo-tools.projectStatus.toggleFeature".to_string(),
                Some(feat.to_string()),
            )
        })
        .collect()
}

// Make sure to keep this up to date with 'TreeItemCollapsibleState'
#[derive(Debug, Clone)]
enum CollapsibleState {
    None = 0,
    Expanded = 2,
}

#[derive(Debug, Clone)]
pub struct OutlineUiRequest {
    pub tx: Sender<Vec<OutlineNodeData>>,
    pub node_type: Option<OutlineNodeType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen]
pub struct OutlineNodeType(OutlineNodeTypeInner);

#[wasm_bindgen]
impl OutlineNodeType {
    #[wasm_bindgen]
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}

impl OutlineNodeType {
    pub fn children(
        &self,
        config: &Config,
        packages: &[Package],
        grouping: Grouping,
        show_features: bool,
    ) -> Vec<OutlineNodeData> {
        use OutlineNodeTypeInner::*;
        match &self.0 {
            Root => OutlineNodeData::root_children(config, packages, grouping, show_features),
            RootFeatures => OutlineNodeData::root_features_children(config, packages),
            Package { name } => try_package(name, packages)
                .map(|p| OutlineNodeData::package_children(config, p, show_features))
                .unwrap_or_default(),
            PackageFeatures { package } => try_package(package, packages)
                .map(|p| OutlineNodeData::package_features_children(config, p))
                .unwrap_or_default(),
            Libraries => OutlineNodeData::targets_children(TargetType::Lib, config, packages),
            Binaries => OutlineNodeData::targets_children(TargetType::Bin, config, packages),
            Examples => OutlineNodeData::targets_children(TargetType::Example, config, packages),
            Benchmarks => OutlineNodeData::targets_children(TargetType::Bench, config, packages),
            // All others never have further child nodes
            RootFeature(_) => Vec::new(),
            Feature { .. } => Vec::new(),
            Lib { .. } => Vec::new(),
            Bin { .. } => Vec::new(),
            Example { .. } => Vec::new(),
            Bench { .. } => Vec::new(),
        }
    }

    pub fn try_into_package(self) -> Option<String> {
        match self.0 {
            OutlineNodeTypeInner::Package { name } => Some(name),
            _ => None,
        }
    }

    pub fn try_into_build_target(self) -> Option<BuildTarget> {
        use OutlineNodeTypeInner::*;
        let build_target = |package, target| {
            Some(BuildTarget {
                package,
                target: Some(target),
            })
        };
        match self.0 {
            Lib { package, name } => build_target(package, BuildSubTarget::Lib(name)),
            Bin { package, name } => build_target(package, BuildSubTarget::Bin(name)),
            Example { package, name } => build_target(package, BuildSubTarget::Example(name)),
            Bench { package, name } => build_target(package, BuildSubTarget::Bench(name)),
            _ => None,
        }
    }

    pub fn try_into_run_target(self) -> Option<RunTarget> {
        use OutlineNodeTypeInner::*;
        let run_target = |package, target| {
            Some(RunTarget {
                package,
                target: Some(target),
            })
        };
        match self.0 {
            Bin { package, name } => run_target(package, RunSubTarget::Bin(name)),
            Example { package, name } => run_target(package, RunSubTarget::Example(name)),
            _ => None,
        }
    }

    pub fn try_into_bench_target(self) -> Option<BenchTarget> {
        use OutlineNodeTypeInner::*;
        match self.0 {
            Bench { package, name } => Some(BenchTarget {
                package,
                target: Some(name),
            }),
            _ => None,
        }
    }
}

fn try_package<'a>(package: &str, packages: &'a [Package]) -> Option<&'a Package> {
    packages.iter().find(|p| p.name == package)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutlineNodeTypeInner {
    Root,
    RootFeatures,
    RootFeature(String),
    Package { name: String },
    PackageFeatures { package: String },
    Feature { package: String, name: String },
    Lib { package: String, name: String },
    Bin { package: String, name: String },
    Example { package: String, name: String },
    Bench { package: String, name: String },
    Libraries,
    Binaries,
    Examples,
    Benchmarks,
}

trait TargetExt {
    fn icon(&self) -> Icon;
    fn label(&self) -> String;
    fn targets_node_type(&self) -> OutlineNodeType;
}

impl TargetExt for TargetType {
    fn icon(&self) -> Icon {
        match self {
            TargetType::Lib => LIB_TARGET,
            TargetType::Bin => BIN_TARGET,
            TargetType::Example => EXAMPLE_TARGET,
            TargetType::Bench => BENCH_TARGET,
        }
    }

    fn label(&self) -> String {
        match self {
            TargetType::Lib => "Libraries".to_string(),
            TargetType::Bin => "Binaries".to_string(),
            TargetType::Example => "Examples".to_string(),
            TargetType::Bench => "Benchmarks".to_string(),
        }
    }

    fn targets_node_type(&self) -> OutlineNodeType {
        match self {
            TargetType::Lib => OutlineNodeType(OutlineNodeTypeInner::Libraries),
            TargetType::Bin => OutlineNodeType(OutlineNodeTypeInner::Binaries),
            TargetType::Example => OutlineNodeType(OutlineNodeTypeInner::Examples),
            TargetType::Bench => OutlineNodeType(OutlineNodeTypeInner::Benchmarks),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutlineNodeData {
    label: String,
    icon: Icon,
    collapsible_state: CollapsibleState,
    node_type: OutlineNodeType,
    context_value: Option<String>,
    tooltip: Option<String>,
    description: Option<String>,
    command: Option<String>,
    command_arg: Option<String>,
}

impl OutlineNodeData {
    pub fn root(label: String, num_targets: usize) -> Self {
        OutlineNodeData {
            label,
            icon: PROJECT,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType(OutlineNodeTypeInner::Root),
            context_value: Some("project".to_string()),
            tooltip: None,
            description: Some(format!("{num_targets} targets")),
            command: None,
            command_arg: None,
        }
    }

    fn root_children(
        config: &Config,
        packages: &[Package],
        grouping: Grouping,
        show_features: bool,
    ) -> Vec<Self> {
        match grouping {
            Grouping::Packages => {
                OutlineNodeData::packages_root_children(config, packages, show_features)
            }
            Grouping::TargetTypes => {
                OutlineNodeData::target_types_root_children(metadata::TargetType::counts(packages))
            }
        }
    }

    fn target_types_root_children(target_counts: HashMap<TargetType, usize>) -> Vec<Self> {
        use metadata::TargetType::*;
        [Lib, Bin, Example, Bench]
            .into_iter()
            .filter_map(|target| {
                target_counts
                    .get(&target)
                    .copied()
                    .map(|num_targets| Self::target(target, num_targets))
            })
            .collect()
    }

    fn target(target: TargetType, num_targets: usize) -> Self {
        Self {
            label: target.label(),
            icon: target.icon(),
            collapsible_state: CollapsibleState::Expanded,
            node_type: target.targets_node_type(),
            context_value: None,
            tooltip: None,
            description: Some(num_targets.to_string()),
            command: None,
            command_arg: None,
        }
    }

    fn targets_children(target: TargetType, config: &Config, packages: &[Package]) -> Vec<Self> {
        packages
            .iter()
            .flat_map(|pkg| {
                pkg.targets
                    .iter()
                    .filter(|t| t.target_type == target)
                    .map(|t| Self::target_leaf(target, config, pkg.name.to_string(), t, true))
            })
            .collect()
    }

    fn target_leaf(
        target_type: TargetType,
        config: &Config,
        package: String,
        target: &Target,
        show_package: bool,
    ) -> Self {
        match target_type {
            TargetType::Lib => Self::lib_target_leaf(config, package, target, show_package),
            TargetType::Bin => Self::bin_target_leaf(config, package, target, show_package),
            TargetType::Example => Self::example_target_leaf(config, package, target, show_package),
            TargetType::Bench => Self::bench_target_leaf(config, package, target, show_package),
        }
    }

    fn lib_target_leaf(
        config: &Config,
        package: String,
        target: &Target,
        show_package: bool,
    ) -> Self {
        let types_str = target.target_kind.iter().map(|k| k.to_string()).join(", ");

        let description = if show_package && target.name != package {
            format!("{} · {}", package, types_str)
        } else {
            types_str
        };

        let mut label = target.name.to_string();
        let mut context = vec!["cargoTarget", "isLibrary", "supportsBuild"];

        if config.selected_package.as_ref() == Some(&package) {
            if let Some(selected_package) = config.package_selection() {
                let target_name = target.name.as_str();
                if selected_package.build_target_matches(TargetType::Lib, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
            } else {
                context.push("canBeSelectedBuildTarget");
            }
        }

        let node_type = OutlineNodeType(OutlineNodeTypeInner::Lib {
            package,
            name: target.name.to_string(),
        });

        Self::leaf(
            label,
            node_type,
            TargetType::Lib.icon(),
            context.join(","),
            Some(description),
            target.source.to_string(),
        )
    }

    fn bin_target_leaf(
        config: &Config,
        package: String,
        target: &Target,
        show_package: bool,
    ) -> Self {
        let mut label = target.name.to_string();
        let mut context = vec![
            "cargoTarget",
            "isExecutable",
            "supportsBuild",
            "supportsRun",
            "supportsDebug",
        ];

        if config.selected_package.as_ref() == Some(&package) {
            if let Some(selected_package) = config.package_selection() {
                let target_name = target.name.as_str();
                if selected_package.build_target_matches(TargetType::Bin, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
                if selected_package.run_target_matches(TargetType::Bin, target_name) {
                    label.push_str(" 🚀");
                    context.push("isSelectedRunTarget");
                } else {
                    context.push("canBeSelectedRunTarget");
                }
            } else {
                context.push("canBeSelectedBuildTarget");
                context.push("canBeSelectedRunTarget");
            }
        }

        let description = if show_package && target.name != package {
            Some(package.clone())
        } else {
            None
        };

        let node_type = OutlineNodeType(OutlineNodeTypeInner::Bin {
            package,
            name: target.name.to_string(),
        });

        Self::leaf(
            label,
            node_type,
            TargetType::Bin.icon(),
            context.join(","),
            description,
            target.source.to_string(),
        )
    }

    fn example_target_leaf(
        config: &Config,
        package: String,
        target: &Target,
        show_package: bool,
    ) -> Self {
        let mut label = target.name.to_string();
        let mut context = vec![
            "cargoTarget",
            "isExample",
            "isExecutable",
            "supportsBuild",
            "supportsRun",
            "supportsDebug",
        ];

        if config.selected_package.as_ref() == Some(&package) {
            let target_name = target.name.as_str();
            if let Some(selected_package) = config.package_selection() {
                if selected_package.build_target_matches(TargetType::Example, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
                if selected_package.run_target_matches(TargetType::Example, target_name) {
                    label.push_str(" 🚀");
                    context.push("isSelectedRunTarget");
                } else {
                    context.push("canBeSelectedRunTarget");
                }
            } else {
                context.push("isSelectedBuildTarget");
                context.push("canBeSelectedRunTarget");
            }
        }

        let description = if show_package && target.name != package {
            Some(package.clone())
        } else {
            None
        };

        let node_type = OutlineNodeType(OutlineNodeTypeInner::Example {
            package,
            name: target.name.to_string(),
        });

        Self::leaf(
            label,
            node_type,
            TargetType::Example.icon(),
            context.join(","),
            description,
            target.source.to_string(),
        )
    }

    fn bench_target_leaf(
        config: &Config,
        package: String,
        target: &Target,
        show_package: bool,
    ) -> Self {
        let mut label = target.name.to_string();
        let mut context = vec!["cargoTarget", "isBench", "supportsBuild", "supportsBench"];

        if config.selected_package.as_ref() == Some(&package) {
            if let Some(selected_package) = config.package_selection() {
                let target_name = target.name.as_str();
                if selected_package.build_target_matches(TargetType::Bench, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
                if selected_package.bench_target_matches(target_name) {
                    label.push_str(" ⚡");
                    context.push("isSelectedBenchmarkTarget");
                } else {
                    context.push("canBeSelectedBenchmarkTarget");
                }
            } else {
                context.push("canBeSelectedBuildTarget");
                context.push("canBeSelectedBenchmarkTarget");
            }
        }

        let description = if show_package && target.name != package {
            Some(package.clone())
        } else {
            None
        };

        let node_type = OutlineNodeType(OutlineNodeTypeInner::Bench {
            package,
            name: target.name.to_string(),
        });

        Self::leaf(
            label,
            node_type,
            TargetType::Bench.icon(),
            context.join(","),
            description,
            target.source.to_string(),
        )
    }

    fn leaf(
        label: String,
        node_type: OutlineNodeType,
        icon: Icon,
        context_value: String,
        description: Option<String>,
        source: String,
    ) -> Self {
        Self {
            label,
            icon,
            collapsible_state: CollapsibleState::None,
            node_type,
            context_value: Some(context_value),
            tooltip: None,
            description,
            command: Some("vscode.open".to_string()),
            command_arg: Some(source),
        }
    }

    fn into_node(self) -> CargoOutlineNode {
        let Self {
            label,
            icon,
            collapsible_state,
            node_type,
            context_value,
            tooltip,
            description,
            command,
            command_arg,
        } = self;

        match node_type.0.clone() {
            OutlineNodeTypeInner::RootFeature(name) => CargoOutlineNode::feature(
                label,
                icon,
                collapsible_state as u32,
                node_type,
                "cargo-tools.projectOutline.toggleFeature".to_string(),
                vec![name],
            ),
            OutlineNodeTypeInner::Feature { package, name } => CargoOutlineNode::feature(
                label,
                icon,
                collapsible_state as u32,
                node_type,
                "cargo-tools.projectOutline.toggleFeature".to_string(),
                vec![package, name],
            ),
            _ => CargoOutlineNode::new(
                label,
                icon,
                collapsible_state as u32,
                node_type,
                context_value,
                description,
                tooltip,
                command,
                command_arg,
            ),
        }
    }

    fn packages_root_children(
        config: &Config,
        packages: &[Package],
        show_features: bool,
    ) -> Vec<Self> {
        let root_features = Self {
            label: "Features".to_string(),
            icon: FEATURES_CONFIG,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType(OutlineNodeTypeInner::RootFeatures),
            context_value: None,
            tooltip: None,
            description: None,
            command: None,
            command_arg: None,
        };

        let packages = packages
            .iter()
            .map(|p| Self::package(config.selected_package.as_deref(), p));

        if show_features {
            iter::once(root_features).chain(packages).collect()
        } else {
            packages.collect()
        }
    }

    fn package(selected_package: Option<&str>, package: &Package) -> Self {
        let mut label = package.name.to_string();
        let mut context = vec!["workspaceMember"];

        if let Some(name) = selected_package
            && name == label
        {
            label.push_str(" 📦");
            context.push("isSelectedPackage");
        } else {
            context.push("canBeSelectedPackage");
        }

        let name = package.name.to_string();

        Self {
            label,
            icon: PACKAGE,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType(OutlineNodeTypeInner::Package { name }),
            context_value: Some(context.join(",")),
            tooltip: None,
            description: Some(format!("{} target(s)", package.targets.len())),
            command: Some("vscode.open".to_string()),
            command_arg: Some(package.manifest.to_string()),
        }
    }

    fn root_features_children(config: &Config, packages: &[Package]) -> Vec<Self> {
        let selected_features = match &config.selected_features {
            Features::All => vec!["All features"],
            Features::Some(items) => items.iter().map(|i| i.as_str()).collect(),
        };

        packages
            .iter()
            .flat_map(|p| &p.features)
            .sorted()
            .unique()
            .map(|feature| {
                let name = feature.to_string();
                let icon = if selected_features.contains(&feature.as_str()) {
                    SELECTED_STATE
                } else {
                    UNSELECTED_STATE
                };

                Self {
                    label: name.clone(),
                    icon,
                    collapsible_state: CollapsibleState::None,
                    node_type: OutlineNodeType(OutlineNodeTypeInner::RootFeature(name)),
                    context_value: None,
                    tooltip: None,
                    description: None,
                    command: None,
                    command_arg: None,
                }
            })
            .collect()
    }

    fn package_features_children(config: &Config, package: &Package) -> Vec<Self> {
        let selected_features = config
            .package_configs
            .get(&package.name)
            .map(|selected| match &selected.selected_features {
                Features::All => vec!["All features"],
                Features::Some(items) => items.iter().map(|i| i.as_str()).collect(),
            })
            .unwrap_or_default();

        package
            .features
            .iter()
            .sorted()
            .map(|feature| {
                let name = feature.clone();
                let package = package.name.clone();
                let icon = if selected_features.contains(&feature.as_str()) {
                    SELECTED_STATE
                } else {
                    UNSELECTED_STATE
                };

                Self {
                    label: feature.clone(),
                    icon,
                    collapsible_state: CollapsibleState::None,
                    node_type: OutlineNodeType(OutlineNodeTypeInner::Feature { package, name }),
                    context_value: None,
                    tooltip: None,
                    description: None,
                    command: None,
                    command_arg: None,
                }
            })
            .collect()
    }

    fn package_children(config: &Config, package: &Package, show_features: bool) -> Vec<Self> {
        let package_name = &package.name;

        let targets = package
            .targets
            .iter()
            .map(|target| match target.target_type {
                TargetType::Lib => {
                    Self::lib_target_leaf(config, package_name.to_string(), target, false)
                }
                TargetType::Bin => {
                    Self::bin_target_leaf(config, package_name.to_string(), target, false)
                }
                TargetType::Example => {
                    Self::example_target_leaf(config, package_name.to_string(), target, false)
                }
                TargetType::Bench => {
                    Self::bench_target_leaf(config, package_name.to_string(), target, false)
                }
            });

        let features = Self {
            label: "Features".to_string(),
            icon: FEATURES_CONFIG,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType(OutlineNodeTypeInner::PackageFeatures {
                package: package_name.clone(),
            }),
            context_value: None,
            tooltip: None,
            description: None,
            command: None,
            command_arg: None,
        };

        if show_features {
            targets.chain(iter::once(features)).collect()
        } else {
            targets.collect()
        }
    }
}

#[wasm_bindgen]
pub struct CargoOutlineTreeProviderHandler {
    tx: Sender<OutlineUiRequest>,
}

#[wasm_bindgen]
impl CargoOutlineTreeProviderHandler {
    pub async fn children(&self, node_type: Option<OutlineNodeType>) -> Vec<CargoOutlineNode> {
        let (tx, mut rx) = channel(1);

        let update = OutlineUiRequest { tx, node_type };

        if let Err(e) = self.tx.clone().send(update).await {
            log_error(&format!("Failed to send UiConfigUpdate: {e}"));
        }

        rx.next()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(OutlineNodeData::into_node)
            .collect()
    }
}

impl CargoOutlineTreeProviderHandler {
    pub fn new(tx: Sender<OutlineUiRequest>) -> Self {
        Self { tx }
    }
}
