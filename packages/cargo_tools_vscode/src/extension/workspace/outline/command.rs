use cargo_tools::cargo::{
    command::{BenchTarget, BuildTarget, RunTarget},
    config::{FeatureTarget, Update},
};
use futures::channel::mpsc::Sender;
use wasm_bindgen_futures::js_sys::Array;

use wasm_bindgen::prelude::*;

use crate::{
    commands::outline::*,
    extension::{
        vscode_task_utils::{CommandBinding, register_commands, take_first, take_first_two},
        workspace::outline::{TargetTypesFilter, treeprovider::OutlineNodeType},
    },
};

#[wasm_bindgen(
    raw_module = "../../../packages/cargo_tools_vscode/src/extension/workspace/outline/command.ts"
)]
extern "C" {
    #[wasm_bindgen]
    fn try_get_node_type(value: Array) -> Option<OutlineNodeType>;
}

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

type CmdFn = fn(Array) -> Option<Command>;

impl Command {
    const fn all() -> [(&'static str, CmdFn); NUMBER_CMDS] {
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

pub fn register_outline_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_commands(tx, Command::all())
}
