use cargo_tools::cargo::selection;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    icon::{
        BENCH_ACTION, BENCH_TARGET, BUILD_ACTION, FEATURES_CONFIG, Icon, PACKAGE, PLATFORM_CONFIG,
        PROFILE_CONFIG, RUN_ACTION, SELECTED_STATE, TARGET_CONFIG, UNSELECTED_STATE,
    },
    vs_code_api::CargoNode,
};

const PACKAGE_CTX: &str = "packageSelection";
const BUILD_CTX: &str = "buildTargetSelection";
const RUN_CTX: &str = "runTargetSelection";
const BENCH_CTX: &str = "benchmarkTargetSelection";

#[wasm_bindgen]
pub struct CargoNodeHandler(CargoNodeInner);

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CargoNodeInner {
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

#[wasm_bindgen]
impl CargoNodeHandler {
    pub fn children(&self, ctx: &CargoConfigurationTreeProviderHandler) -> Vec<CargoNode> {
        let default = "Default".to_string();
        match &self.0 {
            CargoNodeInner::Platform => vec![leaf(
                ctx.selection.platform_target.clone().unwrap_or(default),
                PLATFORM_CONFIG,
                CargoNodeHandler::selection(),
                "Click to select target platform".to_string(),
                "cargo-tools.selectPlatformTarget".to_string(),
                None,
            )],
            CargoNodeInner::BuildConfig => vec![leaf(
                ctx.selection
                    .profile
                    .get_name()
                    .unwrap_or("Default (dev)")
                    .to_string(),
                PROFILE_CONFIG,
                CargoNodeHandler::selection(),
                "Click to select build profile".to_string(),
                "cargo-tools.selectProfile".to_string(),
                None,
            )],
            CargoNodeInner::Package => vec![leaf(
                ctx.selection
                    .package
                    .clone()
                    .unwrap_or("No selection".to_string()),
                PACKAGE,
                CargoNodeHandler::selection(),
                "Click to select package".to_string(),
                "cargo-tools.selectPackage".to_string(),
                None,
            )],
            CargoNodeInner::Target => {
                let build = node(
                    "Build target".to_string(),
                    BUILD_ACTION,
                    CargoNodeHandler::build_target(),
                    Some(BUILD_CTX.to_string()),
                );

                let run = node(
                    "Run target".to_string(),
                    RUN_ACTION,
                    CargoNodeHandler::run_target(),
                    Some(RUN_CTX.to_string()),
                );

                let bench = node(
                    "Benchmark target".to_string(),
                    BENCH_ACTION,
                    CargoNodeHandler::bench_target(),
                    Some(BENCH_CTX.to_string()),
                );

                vec![build, run, bench]
            }
            CargoNodeInner::Features => ctx.feature_leaves(),
            CargoNodeInner::BuildTarget => {
                let build_target = ctx
                    .selection
                    .package_selection()
                    .and_then(|p| p.build_target.clone());
                vec![leaf(
                    build_target
                        .as_ref()
                        .map(|t| t.name().to_string())
                        .clone()
                        .unwrap_or(default),
                    Icon::build_target(&build_target),
                    CargoNodeHandler::selection(),
                    "Click to select build target".to_string(),
                    "cargo-tools.selectBuildTarget".to_string(),
                    None,
                )]
            }
            CargoNodeInner::RunTarget => {
                let run_target = ctx
                    .selection
                    .package_selection()
                    .and_then(|p| p.run_target.clone());
                vec![leaf(
                    run_target
                        .as_ref()
                        .map(|t| t.name().to_string())
                        .clone()
                        .unwrap_or(default),
                    Icon::run_target(&run_target),
                    CargoNodeHandler::selection(),
                    "Click to select run target".to_string(),
                    "cargo-tools.selectRunTarget".to_string(),
                    None,
                )]
            }
            CargoNodeInner::BenchTarget => vec![leaf(
                ctx.selection
                    .package_selection()
                    .and_then(|p| p.benchmark_target.clone())
                    .unwrap_or(default),
                BENCH_TARGET,
                CargoNodeHandler::selection(),
                "Click to select benchmark target".to_string(),
                "cargo-tools.selectBenchmarkTarget".to_string(),
                None,
            )],
            CargoNodeInner::Selection => Vec::new(),
        }
    }
}

fn leaf(
    label: String,
    icon: Icon,
    handler: CargoNodeHandler,
    tooltip: String,
    command: String,
    command_arg: Option<String>,
) -> CargoNode {
    CargoNode::new(
        label,
        icon,
        CollapsibleState::None as u32,
        handler,
        None,
        None,
        Some(tooltip),
        Some(command),
        command_arg,
    )
}

impl CargoNodeHandler {
    fn platform() -> Self {
        Self(CargoNodeInner::Platform)
    }
    fn build_config() -> Self {
        Self(CargoNodeInner::BuildConfig)
    }
    fn package() -> Self {
        Self(CargoNodeInner::Package)
    }
    fn target() -> Self {
        Self(CargoNodeInner::Target)
    }
    fn features() -> Self {
        Self(CargoNodeInner::Features)
    }
    fn build_target() -> Self {
        Self(CargoNodeInner::BuildTarget)
    }
    fn run_target() -> Self {
        Self(CargoNodeInner::RunTarget)
    }
    fn bench_target() -> Self {
        Self(CargoNodeInner::BenchTarget)
    }
    fn selection() -> Self {
        Self(CargoNodeInner::Selection)
    }
}

#[wasm_bindgen]
pub struct CargoConfigurationTreeProviderHandler {
    selection: selection::State,
    available_features: Vec<String>,
}

#[wasm_bindgen]
impl CargoConfigurationTreeProviderHandler {
    pub fn children(&self) -> Vec<CargoNode> {
        let platform = node(
            "Target platform".to_string(),
            PLATFORM_CONFIG,
            CargoNodeHandler::platform(),
            None,
        );
        let build_config = node(
            "Build Configuration".to_string(),
            PROFILE_CONFIG,
            CargoNodeHandler::build_config(),
            None,
        );
        let package = node(
            "Packages".to_string(),
            PACKAGE,
            CargoNodeHandler::package(),
            Some(PACKAGE_CTX.to_string()),
        );
        let target = node(
            "Targets".to_string(),
            TARGET_CONFIG,
            CargoNodeHandler::target(),
            None,
        );
        let feature = node(
            "Features".to_string(),
            FEATURES_CONFIG,
            CargoNodeHandler::features(),
            None,
        );

        vec![platform, build_config, package, target, feature]
    }
}

impl CargoConfigurationTreeProviderHandler {
    pub fn new(selection: selection::State, available_features: Vec<String>) -> Self {
        Self {
            selection,
            available_features,
        }
    }

    fn feature_leaves(&self) -> Vec<CargoNode> {
        self.available_features
            .iter()
            .map(|feat| {
                let selected = match self.selection.selected_features() {
                    selection::Features::All => feat == "All features",
                    selection::Features::Some(selected) => selected.contains(feat),
                };
                let icon = if selected {
                    SELECTED_STATE
                } else {
                    UNSELECTED_STATE
                };

                leaf(
                    feat.to_string(),
                    icon,
                    CargoNodeHandler::selection(),
                    "Click to select target platform".to_string(),
                    "cargo-tools.projectStatus.toggleFeature".to_string(),
                    Some(feat.to_string()),
                )
            })
            .collect()
    }
}

fn node(
    label: String,
    icon: Icon,
    handler: CargoNodeHandler,
    context_value: Option<String>,
) -> CargoNode {
    CargoNode::new(
        label,
        icon,
        CollapsibleState::Expanded as u32,
        handler,
        context_value,
        None,
        None,
        None,
        None,
    )
}

// Make sure to keep this up to date with 'TreeItemCollapsibleState'
enum CollapsibleState {
    None = 0,
    Expanded = 2,
}
