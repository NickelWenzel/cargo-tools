use cargo_metadata::Metadata;
use cargo_tools::cargo::selection::{self};
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    extension::cargo::{Grouping, OutlineSettings, TargetTypesFilter},
    icon::{
        BENCH_ACTION, BENCH_TARGET, BUILD_ACTION, FEATURES_CONFIG, Icon, PACKAGE, PLATFORM_CONFIG,
        PROFILE_CONFIG, RUN_ACTION, SELECTED_STATE, TARGET_CONFIG, UNSELECTED_STATE,
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

        let update = ConfigUiRequest { tx, node_type };

        if let Err(e) = self.tx.clone().send(update).await {
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
                "Click to select target platform".to_string(),
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

#[wasm_bindgen]
pub struct CargoOutlineNodeHandler;

#[wasm_bindgen]
impl CargoOutlineNodeHandler {
    pub fn children(&self, ctx: &CargoOutlineTreeProviderHandler) -> Vec<CargoOutlineNode> {
        todo!()
    }
}

#[wasm_bindgen]
pub struct CargoOutlineTreeProviderHandler {
    metadata: CondensedMetaData,
}

#[wasm_bindgen]
impl CargoOutlineTreeProviderHandler {
    pub fn children(&self) -> Vec<CargoOutlineNode> {
        todo!()
    }
}

impl CargoOutlineTreeProviderHandler {
    pub fn new(metadata: CondensedMetaData) -> Self {
        Self { metadata }
    }
}

#[derive(Default)]
pub struct CondensedMetaData {
    selection: selection::State,
    groups: Groups,
}
enum Groups {
    Packages(Vec<Package>),
    TargetTypes(TargetTypes),
}

impl Groups {
    fn packages(
        metadata: &Metadata,
        package_filter: &str,
        target_types_filter: &TargetTypesFilter,
    ) -> Self {
        let package_filter = package_filter.clone();
        let packages = metadata
            .workspace_packages()
            .into_iter()
            .filter_map(|p| {
                p.name
                    .to_lowercase()
                    .contains(&package_filter.to_lowercase())
                    .then_some(Package::from_package(p))
            })
            .collect();
        Self::Packages(packages)
    }

    fn target_types(
        metadata: &Metadata,
        package_filter: &String,
        target_types_filter: &TargetTypesFilter,
    ) -> Self {
        todo!()
    }
}

impl Default for Groups {
    fn default() -> Self {
        Groups::Packages(Vec::default())
    }
}

enum TargetTypes {
    Lib(TargetType),
    Binaries(TargetType),
    Examples(TargetType),
    Benchmarks(TargetType),
}

struct TargetType {
    package: String,
    name: String,
}

impl CondensedMetaData {
    pub fn new(
        selection: selection::State,
        settings: &OutlineSettings,
        metadata: &Metadata,
    ) -> Self {
        let OutlineSettings {
            package_filter,
            target_types_filter,
            grouping,
        } = settings;

        let groups = match grouping {
            Grouping::Packages => Groups::packages(metadata, package_filter, target_types_filter),
            Grouping::TargetTypes => {
                Groups::target_types(metadata, package_filter, target_types_filter)
            }
        };

        Self { selection, groups }
    }
}

pub struct Package {
    name: String,
    lib: Option<String>,
    binaries: Vec<String>,
    examples: Vec<String>,
    benchmarks: Vec<String>,
    features: Vec<String>,
}

impl Package {
    fn from_package(package: &cargo_metadata::Package) -> Self {
        Self {
            name: package.name.to_string(),
            lib: lib(&package.targets),
            binaries: binaries(&package.targets),
            examples: examples(&package.targets),
            benchmarks: benchmarks(&package.targets),
            features: features(package),
        }
    }
}

fn lib(package: &Vec<cargo_metadata::Target>) -> Option<String> {
    todo!()
}

fn binaries(package: &Vec<cargo_metadata::Target>) -> Vec<String> {
    todo!()
}

fn examples(package: &Vec<cargo_metadata::Target>) -> Vec<String> {
    todo!()
}

fn benchmarks(package: &Vec<cargo_metadata::Target>) -> Vec<String> {
    todo!()
}

fn features(package: &cargo_metadata::Package) -> Vec<String> {
    todo!()
}
