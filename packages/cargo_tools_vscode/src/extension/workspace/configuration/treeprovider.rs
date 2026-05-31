use cargo_tools::cargo::{Config, Features};
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::icon::{
    BENCH_ACTION, BENCH_TARGET, BUILD_ACTION, FEATURES_CONFIG, Icon, PACKAGE, PLATFORM_CONFIG,
    PROFILE_CONFIG, RUN_ACTION, SELECTED_STATE, TARGET_CONFIG, UNSELECTED_STATE,
};
use tracing::error;

#[wasm_bindgen(raw_module = "../configurationTreeProvider.ts")]
extern "C" {
    pub type CargoNode;

    #[wasm_bindgen(constructor)]
    fn new(
        label: String,
        icon: Icon,
        collapsible_state: u32,
        handler: NodeType,
        context_value: Option<String>,
        description: Option<String>,
        tooltip: Option<String>,
        command: Option<String>,
        command_arg: Option<String>,
    ) -> CargoNode;
}

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
            error!("Failed to send UiConfigUpdate: {e}");
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
