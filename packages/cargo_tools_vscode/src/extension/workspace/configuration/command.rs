use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    commands::configuration::*,
    extension::{CommandBinding, CommandMap, register_tasks},
    vs_code_api::{log_error, log_info},
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

type CargoCmdFn = fn(Array) -> Option<Command>;

impl Command {
    pub const fn all() -> [(&'static str, CargoCmdFn); NUMBER_CMDS] {
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

fn take_first<T: DeserializeOwned>(array: Array) -> Option<T> {
    match serde_wasm_bindgen::from_value(array.get(0)) {
        Ok(v) => Some(v),
        Err(e) => {
            log_error(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}

pub fn register_configuration_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, CommandBinding);

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

fn task_map(tx: Sender<Command>) -> CommandMap {
    HashMap::from(
        Command::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    )
}
