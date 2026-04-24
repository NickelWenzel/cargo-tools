use std::collections::HashMap;

use cargo_tools::cargo::{
    command::{BenchTarget, BuildTarget, RunTarget},
    selection::{FeatureType, Update},
};
use futures::{SinkExt, channel::mpsc::Sender};
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    extension::{
        TaskMap, VsCodeTask,
        cargo::{TargetTypesFilter, ui::OutlineNodeType},
        register_tasks,
    },
    vs_code_api::{log, try_get_node_type},
};

pub mod process;

#[derive(Debug, Clone)]
pub enum Command {
    SelectProfile,
    SelectPackage,
    SelectBuildTarget,
    SelectRunTarget,
    SelectBenchmarkTarget,
    SelectPlatformTarget,
    SelectFeatures,
    InstallPlatformTarget,
    SetRustAnalyzerCheckTargets,
    BuildDocs,
    Refresh,
    Clean,
    Build,
    Run,
    Debug,
    Test,
    Bench,
    ToggleFeature(String),
    ProjectOutline(ProjectOutline),
}

pub type CargoCmdFn = fn(Array) -> Option<Command>;

impl Command {
    pub const fn all() -> [(&'static str, CargoCmdFn); 43] {
        use ProjectOutline as PO;
        [
            ("cargo-tools.selectProfile", |_| Some(Self::SelectProfile)),
            ("cargo-tools.selectPackage", |_| Some(Self::SelectPackage)),
            ("cargo-tools.selectBuildTarget", |_| {
                Some(Self::SelectBuildTarget)
            }),
            ("cargo-tools.selectRunTarget", |_| {
                Some(Self::SelectRunTarget)
            }),
            ("cargo-tools.selectBenchmarkTarget", |_| {
                Some(Self::SelectBenchmarkTarget)
            }),
            ("cargo-tools.selectPlatformTarget", |_| {
                Some(Self::SelectPlatformTarget)
            }),
            ("cargo-tools.installPlatformTarget", |_| {
                Some(Self::InstallPlatformTarget)
            }),
            ("cargo-tools.setRustAnalyzerCheckTargets", |_| {
                Some(Self::SetRustAnalyzerCheckTargets)
            }),
            ("cargo-tools.buildDocs", |_| Some(Self::BuildDocs)),
            ("cargo-tools.selectFeatures", |_| Some(Self::SelectFeatures)),
            ("cargo-tools.refresh", |_| Some(Self::Refresh)),
            ("cargo-tools.clean", |_| Some(Self::Clean)),
            ("cargo-tools.projectStatus.build", |_| Some(Self::Build)),
            ("cargo-tools.projectStatus.run", |_| Some(Self::Run)),
            ("cargo-tools.projectStatus.debug", |_| Some(Self::Debug)),
            ("cargo-tools.projectStatus.test", |_| Some(Self::Test)),
            ("cargo-tools.projectStatus.bench", |_| Some(Self::Bench)),
            ("cargo-tools.projectStatus.toggleFeature", |arg| {
                take_first(arg).map(Self::ToggleFeature)
            }),
            ("cargo-tools.projectOutline.selectPackage", |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(Update::SelectedPackage)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.unselectPackage", |_| {
                Some(PO::Select(Update::SelectedPackage(None)).to_cmd())
            }),
            ("cargo-tools.projectOutline.setBuildTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_build_target)
                    .map(|build_target| build_target.target)
                    .map(Update::SelectedBuildTarget)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.unsetBuildTarget", |_| {
                Some(PO::Select(Update::SelectedBuildTarget(None)).to_cmd())
            }),
            ("cargo-tools.projectOutline.setRunTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(|run_target| run_target.target)
                    .map(Update::SelectedRunTarget)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.unsetRunTarget", |_| {
                Some(PO::Select(Update::SelectedRunTarget(None)).to_cmd())
            }),
            ("cargo-tools.projectOutline.setBenchmarkTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_bench_target)
                    .map(|bench_target| bench_target.target)
                    .map(Update::SelectedBenchmarkTarget)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.unsetBenchmarkTarget", |_| {
                Some(PO::Select(Update::SelectedBenchmarkTarget(None)).to_cmd())
            }),
            ("cargo-tools.projectOutline.buildWorkspace", |_| {
                Some(PO::Build(None).to_cmd())
            }),
            ("cargo-tools.projectOutline.testWorkspace", |_| {
                Some(PO::Test(None).to_cmd())
            }),
            ("cargo-tools.projectOutline.cleanWorkspace", |_| {
                Some(PO::Clean(None).to_cmd())
            }),
            ("cargo-tools.projectOutline.buildPackage", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_package)
                    .map(|p| Some(BuildTarget::package_only(p)))
                    .map(PO::Build)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.testPackage", |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(PO::Test)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.cleanPackage", |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(PO::Clean)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.buildTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_build_target)
                    .map(|t| PO::Build(Some(t)))
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.runTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(PO::Run)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.debugTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(PO::Debug)
                    .map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.benchTarget", |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_bench_target)
                    .map(PO::Bench)
                    .map(PO::to_cmd)
            }),
            (
                "cargo-tools.projectOutline.setWorkspaceMemberFilter",
                |_| Some(PO::SelectWorkspaceMemberFilter.to_cmd()),
            ),
            (
                "cargo-tools.projectOutline.editWorkspaceMemberFilter",
                |arg| PO::from_str(PO::EditWorkspaceMemberFilter, arg).map(PO::to_cmd),
            ),
            ("cargo-tools.projectOutline.showTargetTypeFilter", |_| {
                Some(PO::SelectTargetTypeFilter.to_cmd())
            }),
            (
                "cargo-tools.projectOutline.editWorkspaceMemberFilter",
                |arg| {
                    PO::from_target_types_filter_update(PO::EditTargetTypeFilter, arg)
                        .map(PO::to_cmd)
                },
            ),
            ("cargo-tools.projectOutline.clearAllFilters", |_| {
                Some(PO::ClearAllFilters.to_cmd())
            }),
            (
                "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping",
                |_| Some(PO::ToggleWorkspaceMemberGrouping.to_cmd()),
            ),
            (
                "cargo-tools.projectOutline.toggleFeature",
                |arg| match arg.length() {
                    1 => take_first(arg)
                        .map(|feature| PO::ToggleFeature {
                            feature_type: FeatureType::Package(None),
                            feature,
                        })
                        .map(PO::to_cmd),
                    2 => take_first_two(arg)
                        .map(|(package, feature)| PO::ToggleFeature {
                            feature_type: FeatureType::Package(Some(package)),
                            feature,
                        })
                        .map(PO::to_cmd),
                    _ => None,
                },
            ),
        ]
    }
}

#[derive(Debug, Clone)]
pub enum ProjectOutline {
    Select(Update),
    Unselect(Update),
    Build(Option<BuildTarget>),
    Test(Option<String>),
    Clean(Option<String>),
    Run(RunTarget),
    Debug(RunTarget),
    Bench(BenchTarget),
    SelectWorkspaceMemberFilter,
    EditWorkspaceMemberFilter(String),
    SelectTargetTypeFilter,
    EditTargetTypeFilter(TargetTypesFilter),
    ClearAllFilters,
    ToggleWorkspaceMemberGrouping,
    ToggleFeature {
        feature_type: FeatureType,
        feature: String,
    },
}

impl ProjectOutline {
    pub const fn to_cmd(self) -> Command {
        Command::ProjectOutline(self)
    }

    pub fn from_update(cmd: fn(Update) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_build_target(cmd: fn(Option<BuildTarget>) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_run_target(cmd: fn(RunTarget) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_optional_str(cmd: fn(Option<String>) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_str(cmd: fn(String) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_target_types_filter_update(
        cmd: fn(TargetTypesFilter) -> Self,
        arg: Array,
    ) -> Option<Self> {
        take_first(arg).map(cmd)
    }
}

fn take_first<T: DeserializeOwned>(array: Array) -> Option<T> {
    match serde_wasm_bindgen::from_value(array.get(0)) {
        Ok(v) => Some(v),
        Err(e) => {
            log(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}

fn take_first_two<T: DeserializeOwned, U: DeserializeOwned>(array: Array) -> Option<(T, U)> {
    match (
        serde_wasm_bindgen::from_value(array.get(0)),
        serde_wasm_bindgen::from_value(array.get(1)),
    ) {
        (Ok(v_0), Ok(v_1)) => Some((v_0, v_1)),
        (Err(e_0), Err(e_1)) => {
            log(&format!("Failed to deserialize update: {e_0}, {e_1}"));
            None
        }
        (Err(e), _) | (_, Err(e)) => {
            log(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}

pub fn register_cargo_commands(tx: Sender<Command>) -> Vec<VsCodeTask> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, VsCodeTask);

fn create_vs_code_command(
    tx: Sender<Command>,
    key: &'static str,
    cargo_cmd_fn: CargoCmdFn,
) -> CmdKeyValuePair {
    let cmd = Closure::new(move |args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            let Some(cmd) = cargo_cmd_fn(args) else {
                log("Failed to extract cargo command");
                return;
            };
            log(&format!("Sending VS Code cargo command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    });

    (key, cmd)
}

pub fn task_map(tx: Sender<Command>) -> TaskMap {
    HashMap::from(
        Command::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    )
}
