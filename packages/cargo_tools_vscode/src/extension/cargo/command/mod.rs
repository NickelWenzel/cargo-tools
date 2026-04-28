use std::collections::HashMap;

use cargo_tools::cargo::{
    command::{BenchTarget, BuildTarget, RunTarget},
    config::{FeatureTarget, Update},
};
use futures::{SinkExt, channel::mpsc::Sender};
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    commands::cargo::*,
    extension::{
        TaskMap, VsCodeTask,
        cargo::{TargetTypesFilter, ui::OutlineNodeType},
        register_tasks,
    },
    vs_code_api::{log_error, log_info, try_get_node_type},
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
    pub const fn all() -> [(&'static str, CargoCmdFn); NUMBER_CMDS] {
        use ProjectOutline as PO;
        [
            (CARGO_TOOLS_SELECT_PROFILE, |_| Some(Self::SelectProfile)),
            (CARGO_TOOLS_SELECT_PACKAGE, |_| Some(Self::SelectPackage)),
            (CARGO_TOOLS_SELECT_BUILD_TARGET, |_| {
                Some(Self::SelectBuildTarget)
            }),
            (CARGO_TOOLS_SELECT_RUN_TARGET, |_| {
                Some(Self::SelectRunTarget)
            }),
            (CARGO_TOOLS_SELECT_BENCHMARK_TARGET, |_| {
                Some(Self::SelectBenchmarkTarget)
            }),
            (CARGO_TOOLS_SELECT_PLATFORM_TARGET, |_| {
                Some(Self::SelectPlatformTarget)
            }),
            (CARGO_TOOLS_INSTALL_PLATFORM_TARGET, |_| {
                Some(Self::InstallPlatformTarget)
            }),
            (CARGO_TOOLS_SET_RUST_ANALYZER_CHECK_TARGETS, |_| {
                Some(Self::SetRustAnalyzerCheckTargets)
            }),
            (CARGO_TOOLS_BUILD_DOCS, |_| Some(Self::BuildDocs)),
            (CARGO_TOOLS_SELECT_FEATURES, |_| Some(Self::SelectFeatures)),
            (CARGO_TOOLS_REFRESH, |_| Some(Self::Refresh)),
            (CARGO_TOOLS_CLEAN, |_| Some(Self::Clean)),
            (CARGO_TOOLS_PROJECT_STATUS_BUILD, |_| Some(Self::Build)),
            (CARGO_TOOLS_PROJECT_STATUS_RUN, |_| Some(Self::Run)),
            (CARGO_TOOLS_PROJECT_STATUS_DEBUG, |_| Some(Self::Debug)),
            (CARGO_TOOLS_PROJECT_STATUS_TEST, |_| Some(Self::Test)),
            (CARGO_TOOLS_PROJECT_STATUS_BENCH, |_| Some(Self::Bench)),
            (CARGO_TOOLS_PROJECT_STATUS_TOGGLE_FEATURE, |arg| {
                take_first(arg).map(Self::ToggleFeature)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SELECT_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(Update::SelectedPackage)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSELECT_PACKAGE, |_| {
                Some(PO::Select(Update::SelectedPackage(None)).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SET_BUILD_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_build_target)
                    .map(|build_target| build_target.target)
                    .map(Update::SelectedBuildTarget)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSET_BUILD_TARGET, |_| {
                Some(PO::Select(Update::SelectedBuildTarget(None)).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SET_RUN_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(|run_target| run_target.target)
                    .map(Update::SelectedRunTarget)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSET_RUN_TARGET, |_| {
                Some(PO::Select(Update::SelectedRunTarget(None)).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SET_BENCHMARK_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_bench_target)
                    .map(|bench_target| bench_target.target)
                    .map(Update::SelectedBenchmarkTarget)
                    .map(PO::Select)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSET_BENCHMARK_TARGET, |_| {
                Some(PO::Select(Update::SelectedBenchmarkTarget(None)).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BUILD_WORKSPACE, |_| {
                Some(PO::Build(None).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_TEST_WORKSPACE, |_| {
                Some(PO::Test(None).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_CLEAN_WORKSPACE, |_| {
                Some(PO::Clean(None).to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BUILD_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_package)
                    .map(|p| Some(BuildTarget::package_only(p)))
                    .map(PO::Build)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_TEST_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(PO::Test)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_CLEAN_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(PO::Clean)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BUILD_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_build_target)
                    .map(|t| PO::Build(Some(t)))
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_RUN_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(PO::Run)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_DEBUG_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(PO::Debug)
                    .map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BENCH_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_bench_target)
                    .map(PO::Bench)
                    .map(PO::to_cmd)
            }),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_SET_WORKSPACE_MEMBER_FILTER,
                |_| Some(PO::SelectWorkspaceMemberFilter.to_cmd()),
            ),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_EDIT_WORKSPACE_MEMBER_FILTER,
                |arg| PO::from_str(PO::EditWorkspaceMemberFilter, arg).map(PO::to_cmd),
            ),
            (CARGO_TOOLS_PROJECT_OUTLINE_SHOW_TARGET_TYPE_FILTER, |_| {
                Some(PO::SelectTargetTypeFilter.to_cmd())
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_EDIT_TARGET_TYPE_FILTER, |arg| {
                PO::from_target_types_filter_update(PO::EditTargetTypeFilter, arg).map(PO::to_cmd)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_CLEAR_ALL_FILTERS, |_| {
                Some(PO::ClearAllFilters.to_cmd())
            }),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_TOGGLE_WORKSPACE_MEMBER_GROUPING,
                |_| Some(PO::ToggleWorkspaceMemberGrouping.to_cmd()),
            ),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_TOGGLE_FEATURE,
                |arg| match arg.length() {
                    1 => take_first(arg)
                        .map(|feature| PO::ToggleFeature {
                            feature_type: FeatureTarget::Workspace,
                            feature,
                        })
                        .map(PO::to_cmd),
                    2 => take_first_two(arg)
                        .map(|(package, feature)| PO::ToggleFeature {
                            feature_type: FeatureTarget::Package(package),
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
        feature_type: FeatureTarget,
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
            log_error(&format!("Failed to deserialize update: {e}"));
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
            log_error(&format!("Failed to deserialize update: {e_0}, {e_1}"));
            None
        }
        (Err(e), _) | (_, Err(e)) => {
            log_error(&format!("Failed to deserialize update: {e}"));
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
                log_error("Failed to extract cargo command");
                return;
            };
            log_info(&format!("Sending VS Code cargo command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log_error(&format!("Failed to queue msg: {}", e));
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
