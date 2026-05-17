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
    commands::outline::*,
    extension::{
        CommandBinding, CommandMap, register_tasks,
        workspace::outline::{TargetTypesFilter, treeprovider::OutlineNodeType},
    },
    vs_code_api::{log_error, log_info, try_get_node_type},
};

#[derive(Debug, Clone)]
pub enum Command {
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

type OutlineCmdFn = fn(Array) -> Option<Command>;

impl Command {
    const fn all() -> [(&'static str, OutlineCmdFn); NUMBER_CMDS] {
        [
            (CARGO_TOOLS_PROJECT_OUTLINE_SELECT_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(Update::SelectedPackage)
                    .map(Self::Select)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSELECT_PACKAGE, |_| {
                Some(Self::Select(Update::SelectedPackage(None)))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SET_BUILD_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_build_target)
                    .map(|build_target| build_target.target)
                    .map(Update::SelectedBuildTarget)
                    .map(Self::Select)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSET_BUILD_TARGET, |_| {
                Some(Self::Select(Update::SelectedBuildTarget(None)))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SET_RUN_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(|run_target| run_target.target)
                    .map(Update::SelectedRunTarget)
                    .map(Self::Select)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSET_RUN_TARGET, |_| {
                Some(Self::Select(Update::SelectedRunTarget(None)))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_SET_BENCHMARK_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_bench_target)
                    .map(|bench_target| bench_target.target)
                    .map(Update::SelectedBenchmarkTarget)
                    .map(Self::Select)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_UNSET_BENCHMARK_TARGET, |_| {
                Some(Self::Select(Update::SelectedBenchmarkTarget(None)))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BUILD_WORKSPACE, |_| {
                Some(Self::Build(None))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_TEST_WORKSPACE, |_| {
                Some(Self::Test(None))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_CLEAN_WORKSPACE, |_| {
                Some(Self::Clean(None))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BUILD_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_package)
                    .map(|p| Some(BuildTarget::package_only(p)))
                    .map(Self::Build)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_TEST_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(Self::Test)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_CLEAN_PACKAGE, |arg| {
                try_get_node_type(arg)
                    .map(OutlineNodeType::try_into_package)
                    .map(Self::Clean)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BUILD_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_build_target)
                    .map(|t| Self::Build(Some(t)))
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_RUN_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(Self::Run)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_DEBUG_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_run_target)
                    .map(Self::Debug)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_BENCH_TARGET, |arg| {
                try_get_node_type(arg)
                    .and_then(OutlineNodeType::try_into_bench_target)
                    .map(Self::Bench)
            }),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_SET_WORKSPACE_MEMBER_FILTER,
                |_| Some(Self::SelectWorkspaceMemberFilter),
            ),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_EDIT_WORKSPACE_MEMBER_FILTER,
                |_| Some(Self::Select(Update::SelectedBuildTarget(None))),
            ),
            (CARGO_TOOLS_PROJECT_OUTLINE_SHOW_TARGET_TYPE_FILTER, |_| {
                Some(Self::SelectTargetTypeFilter)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_EDIT_TARGET_TYPE_FILTER, |arg| {
                Self::from_target_types_filter_update(Self::EditTargetTypeFilter, arg)
            }),
            (CARGO_TOOLS_PROJECT_OUTLINE_CLEAR_ALL_FILTERS, |_| {
                Some(Self::ClearAllFilters)
            }),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_TOGGLE_WORKSPACE_MEMBER_GROUPING,
                |_| Some(Self::ToggleWorkspaceMemberGrouping),
            ),
            (
                CARGO_TOOLS_PROJECT_OUTLINE_TOGGLE_FEATURE,
                |arg| match arg.length() {
                    1 => take_first(arg).map(|feature| Self::ToggleFeature {
                        feature_type: FeatureTarget::Workspace,
                        feature,
                    }),
                    2 => take_first_two(arg).map(|(package, feature)| Self::ToggleFeature {
                        feature_type: FeatureTarget::Package(package),
                        feature,
                    }),
                    _ => None,
                },
            ),
        ]
    }

    fn from_target_types_filter_update(
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

pub fn register_outline_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, CommandBinding);

fn create_vs_code_command(
    tx: Sender<Command>,
    key: &'static str,
    cargo_cmd_fn: OutlineCmdFn,
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

fn task_map(tx: Sender<Command>) -> CommandMap {
    HashMap::from(
        Command::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    )
}
