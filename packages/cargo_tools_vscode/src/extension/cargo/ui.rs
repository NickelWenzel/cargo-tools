use std::{collections::HashMap, iter};

use cargo_tools::cargo::{
    metadata::{self, CondensedPackage, CondensedTarget, Target},
    selection::{self},
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
    vs_code_api::{CargoNode, CargoOutlineNode, log},
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
    pub fn children(
        &self,
        selection: &selection::State,
        available_features: &[String],
    ) -> Vec<NodeData> {
        let default = "Default".to_string();
        match &self.0 {
            NodeTypeInner::Platform => vec![NodeData::leaf(
                selection.platform_target.clone().unwrap_or(default),
                PLATFORM_CONFIG,
                NodeType::selection(),
                "Click to select target platform".to_string(),
                "cargo-tools.selectPlatformTarget".to_string(),
                None,
            )],
            NodeTypeInner::BuildConfig => vec![NodeData::leaf(
                selection
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
                selection
                    .package
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
            NodeTypeInner::Features => feature_leaves(selection, available_features),
            NodeTypeInner::BuildTarget => {
                let build_target = selection
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
                let run_target = selection
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
                selection
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
            log(&format!("Failed to send UiConfigUpdate: {e}"));
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

fn feature_leaves(selection: &selection::State, available_features: &[String]) -> Vec<NodeData> {
    available_features
        .iter()
        .map(|feat| {
            let selected = match selection.selected_features() {
                selection::Features::All => feat == "All features",
                selection::Features::Some(selected) => selected.contains(feat),
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
    pub fn is_feature(&self) -> bool {
        matches!(self, &Self(OutlineNodeTypeInner::FeatureLeaf))
    }
}

impl OutlineNodeType {
    pub fn children(
        &self,
        selection: &selection::State,
        packages: &[CondensedPackage],
    ) -> Vec<OutlineNodeData> {
        match &self.0 {
            OutlineNodeTypeInner::Packages(packages_type) => match packages_type {
                Packages::Root => OutlineNodeData::packages_root_children(selection, packages),
                Packages::Features(features) => match features {
                    Features::Root => vec![OutlineNodeData::all_features(selection)],
                    Features::Package(package) => try_package(package, packages)
                        .map(|p| OutlineNodeData::package_features(selection, p))
                        .unwrap_or_default(),
                },
                Packages::Package(package) => try_package(package, packages)
                    .map(|p| OutlineNodeData::package_children(selection, p))
                    .unwrap_or_default(),
            },
            OutlineNodeTypeInner::Targets(targets) => match targets {
                Targets::Root => {
                    OutlineNodeData::target_root_children(metadata::Target::counts(packages))
                }
                Targets::Package(target) => {
                    OutlineNodeData::targets_children(*target, selection, packages)
                }
            },
            OutlineNodeTypeInner::Leaf => Vec::new(),
            OutlineNodeTypeInner::FeatureLeaf => Vec::new(),
        }
    }

    fn root(grouping: Grouping) -> Self {
        let inner = match grouping {
            Grouping::Packages => OutlineNodeTypeInner::Packages(Packages::Root),
            Grouping::TargetTypes => OutlineNodeTypeInner::Targets(Targets::Root),
        };

        Self(inner)
    }

    fn targets(targets: Targets) -> Self {
        Self(OutlineNodeTypeInner::Targets(targets))
    }

    fn leaf() -> Self {
        Self(OutlineNodeTypeInner::Leaf)
    }

    fn feature_leaf() -> Self {
        Self(OutlineNodeTypeInner::FeatureLeaf)
    }

    fn features(features: Features) -> Self {
        Self(OutlineNodeTypeInner::Packages(Packages::Features(features)))
    }
}

fn try_package<'a>(
    package: &str,
    packages: &'a [CondensedPackage],
) -> Option<&'a CondensedPackage> {
    packages.iter().find(|p| p.name == package)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutlineNodeTypeInner {
    Packages(Packages),
    Targets(Targets),
    Leaf,
    FeatureLeaf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Packages {
    Root,
    Features(Features),
    Package(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Features {
    Root,
    Package(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Targets {
    Root,
    Package(Target),
}

impl Targets {
    fn label(&self) -> String {
        match self {
            Targets::Root => "Targets".to_string(),
            Targets::Package(target) => match target {
                Target::Lib => "Libraries".to_string(),
                Target::Bin => "Binaries".to_string(),
                Target::Example => "Examples".to_string(),
                Target::Bench => "Benchmarks".to_string(),
            },
        }
    }

    fn icon(&self) -> Icon {
        match self {
            Targets::Root => PROJECT,
            Targets::Package(target) => match target {
                Target::Lib => LIB_TARGET,
                Target::Bin => BIN_TARGET,
                Target::Example => EXAMPLE_TARGET,
                Target::Bench => BENCH_TARGET,
            },
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
    package: Option<String>,
    target: Option<String>,
}

impl OutlineNodeData {
    pub fn root(label: String, grouping: Grouping, num_targets: usize) -> Self {
        OutlineNodeData {
            label,
            icon: PROJECT,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType::root(grouping),
            context_value: Some("project".to_string()),
            tooltip: None,
            description: Some(format!("{num_targets} targets")),
            command: None,
            command_arg: None,
            package: None,
            target: None,
        }
    }

    fn target_root_children(target_counts: HashMap<Target, usize>) -> Vec<Self> {
        use metadata::Target::*;
        [Lib, Bin, Example, Bench]
            .iter()
            .filter_map(|target| {
                target_counts
                    .get(target)
                    .copied()
                    .map(|num_targets| Self::target(Targets::Package(*target), num_targets))
            })
            .collect()
    }

    fn target(target: Targets, num_targets: usize) -> Self {
        Self {
            label: target.label(),
            icon: target.icon(),
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType::targets(target),
            context_value: None,
            tooltip: None,
            description: Some(num_targets.to_string()),
            command: None,
            command_arg: None,
            package: None,
            target: None,
        }
    }

    fn targets_children(
        target: Target,
        selection: &selection::State,
        packages: &[CondensedPackage],
    ) -> Vec<Self> {
        packages
            .iter()
            .flat_map(|pkg| {
                pkg.targets
                    .iter()
                    .filter(|t| t.target_type == target)
                    .map(|t| Self::target_leaf(target, selection, pkg.name.to_string(), t))
            })
            .collect()
    }

    fn target_leaf(
        target_type: Target,
        selection: &selection::State,
        package: String,
        target: &CondensedTarget,
    ) -> Self {
        match target_type {
            Target::Lib => Self::lib_target_leaf(selection, package, target),
            Target::Bin => Self::bin_target_leaf(selection, package, target),
            Target::Example => Self::example_target_leaf(selection, package, target),
            Target::Bench => Self::bench_target_leaf(selection, package, target),
        }
    }

    fn lib_target_leaf(
        selection: &selection::State,
        package: String,
        target: &CondensedTarget,
    ) -> Self {
        let description = target
            .original_types
            .iter()
            .map(|k| k.to_string())
            .join(", ");

        let mut label = target.name.to_string();
        let mut context = vec!["cargoTarget", "isLibrary", "supportsBuild"];

        if selection.package.as_ref() == Some(&package) {
            if let Some(selected_package) = selection.package_selection() {
                let target_name = target.name.as_str();
                if selected_package.build_target_matches(Target::Lib, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
            } else {
                context.push("canBeSelectedBuildTarget");
            }
        }

        Self::leaf(
            label,
            target.name.to_string(),
            package,
            Targets::Package(Target::Lib).icon(),
            context.join(","),
            Some(description),
            target.source.to_string(),
        )
    }

    fn bin_target_leaf(
        selection: &selection::State,
        package: String,
        target: &CondensedTarget,
    ) -> Self {
        let mut label = target.name.to_string();
        let mut context = vec![
            "cargoTarget",
            "isExecutable",
            "supportsBuild",
            "supportsRun",
            "supportsDebug",
        ];

        if selection.package.as_ref() == Some(&package) {
            if let Some(selected_package) = selection.package_selection() {
                let target_name = target.name.as_str();
                if selected_package.build_target_matches(Target::Bin, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
                if selected_package.run_target_matches(Target::Bin, target_name) {
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

        Self::leaf(
            label,
            target.name.to_string(),
            package,
            Targets::Package(Target::Bin).icon(),
            context.join(","),
            None,
            target.source.to_string(),
        )
    }

    fn example_target_leaf(
        selection: &selection::State,
        package: String,
        target: &CondensedTarget,
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

        if selection.package.as_ref() == Some(&package) {
            let target_name = target.name.as_str();
            if let Some(selected_package) = selection.package_selection() {
                if selected_package.build_target_matches(Target::Example, target_name) {
                    label.push_str(" 🔨");
                    context.push("isSelectedBuildTarget");
                } else {
                    context.push("canBeSelectedBuildTarget");
                }
                if selected_package.run_target_matches(Target::Example, target_name) {
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

        Self::leaf(
            label,
            target.name.to_string(),
            package,
            Targets::Package(Target::Example).icon(),
            context.join(","),
            None,
            target.source.to_string(),
        )
    }

    fn bench_target_leaf(
        selection: &selection::State,
        package: String,
        target: &CondensedTarget,
    ) -> Self {
        let mut label = target.name.to_string();
        let mut context = vec!["cargoTarget", "isBench", "supportsBuild", "supportsBench"];

        if selection.package.as_ref() == Some(&package) {
            if let Some(selected_package) = selection.package_selection() {
                let target_name = target.name.as_str();
                if selected_package.build_target_matches(Target::Bench, target_name) {
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

        Self::leaf(
            label,
            target.name.to_string(),
            package,
            Targets::Package(Target::Bench).icon(),
            context.join(","),
            None,
            target.source.to_string(),
        )
    }

    fn leaf(
        label: String,
        target: String,
        package: String,
        icon: Icon,
        context_value: String,
        description: Option<String>,
        source: String,
    ) -> Self {
        Self {
            label,
            icon,
            collapsible_state: CollapsibleState::None,
            node_type: OutlineNodeType::leaf(),
            context_value: Some(context_value),
            tooltip: None,
            description,
            command: Some("vscode.open".to_string()),
            command_arg: Some(source),
            package: Some(package),
            target: Some(target),
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
            package,
            target,
        } = self;

        if node_type == OutlineNodeType::feature_leaf() {
            return CargoOutlineNode::feature(
                label,
                icon,
                collapsible_state as u32,
                node_type,
                "cargo-tools.projectStatus.toggleFeature".to_string(),
                package.into_iter().chain(target).collect(),
            );
        }
        CargoOutlineNode::new(
            label,
            icon,
            collapsible_state as u32,
            node_type,
            context_value,
            description,
            tooltip,
            command,
            command_arg,
            package,
            target,
        )
    }

    fn packages_root_children(
        selection: &selection::State,
        packages: &[CondensedPackage],
    ) -> Vec<OutlineNodeData> {
        let packages = packages
            .iter()
            .map(|p| Self::package(selection.package.as_deref(), p));
        iter::once(Self::features(Features::Root))
            .chain(packages)
            .collect()
    }

    fn package(selected_package: Option<&str>, package: &CondensedPackage) -> OutlineNodeData {
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

        Self {
            label,
            icon: PACKAGE,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType(OutlineNodeTypeInner::Packages(Packages::Package(
                package.name.to_string(),
            ))),
            context_value: Some(context.join(",")),
            tooltip: None,
            description: Some(format!("{} target(s)", package.targets.len())),
            command: Some("vscode.open".to_string()),
            command_arg: Some(package.manifest.to_string()),
            package: None,
            target: None,
        }
    }

    fn features(features: Features) -> OutlineNodeData {
        let package = if let Features::Package(package) = &features {
            Some(package.clone())
        } else {
            None
        };

        Self {
            label: "Features".to_string(),
            icon: FEATURES_CONFIG,
            collapsible_state: CollapsibleState::Expanded,
            node_type: OutlineNodeType::features(features),
            context_value: None,
            tooltip: None,
            description: None,
            command: None,
            command_arg: None,
            package,
            target: None,
        }
    }

    fn all_features(selection: &selection::State) -> Self {
        let feature = "All features";
        let selected = selection.features == selection::Features::All;
        Self::feature(feature, selected, None)
    }

    fn package_features(selection: &selection::State, package: &CondensedPackage) -> Vec<Self> {
        let selected_features = selection
            .package_selection
            .get(&package.name)
            .map(|selected| match &selected.features {
                selection::Features::All => vec!["All features"],
                selection::Features::Some(items) => items.iter().map(|i| i.as_str()).collect(),
            })
            .unwrap_or_default();

        package
            .features
            .iter()
            .map(|f| {
                Self::feature(
                    f,
                    selected_features.contains(&f.as_str()),
                    Some(package.name.as_str()),
                )
            })
            .collect()
    }

    fn package_children(selection: &selection::State, package: &CondensedPackage) -> Vec<Self> {
        let package_name = &package.name;
        let features = Self::features(Features::Package(package_name.clone()));
        let targets = package
            .targets
            .iter()
            .map(|target| match target.target_type {
                Target::Lib => Self::lib_target_leaf(selection, package_name.to_string(), target),
                Target::Bin => Self::bin_target_leaf(selection, package_name.to_string(), target),
                Target::Example => {
                    Self::example_target_leaf(selection, package_name.to_string(), target)
                }
                Target::Bench => {
                    Self::bench_target_leaf(selection, package_name.to_string(), target)
                }
            });

        iter::once(features).chain(targets).collect()
    }

    fn feature(feature: &str, selected: bool, package: Option<&str>) -> Self {
        let icon = if selected {
            SELECTED_STATE
        } else {
            UNSELECTED_STATE
        };

        Self {
            label: feature.to_string(),
            icon,
            collapsible_state: CollapsibleState::None,
            node_type: OutlineNodeType::feature_leaf(),
            context_value: None,
            tooltip: None,
            description: None,
            command: None,
            command_arg: None,
            package: package.map(|p| p.to_string()),
            target: Some(feature.to_string()),
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
            log(&format!("Failed to send UiConfigUpdate: {e}"));
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
