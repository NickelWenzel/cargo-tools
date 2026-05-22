use futures::channel::mpsc::Sender;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::configuration::*,
    extension::vscode_task_utils::{CommandBinding, register_commands, take_first},
};

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
}

type CmdFn = fn(Array) -> Option<Command>;

impl Command {
    pub const fn all() -> [(&'static str, CmdFn); NUMBER_CMDS] {
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
        ]
    }
}

pub fn register_configuration_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_commands(tx, Command::all())
}
