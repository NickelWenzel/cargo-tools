use futures::channel::mpsc::Sender;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::xtask::*,
    extension::vscode_task_utils::{CommandBinding, register_commands},
};

#[wasm_bindgen(raw_module = "../xtaskTreeProvider.ts")]
extern "C" {
    #[wasm_bindgen]
    fn try_get_xtask_label(value: Array) -> Option<String>;
}

#[derive(Debug, Clone)]
pub enum Command {
    RunAlias(String),
    RunAliasWithArgs(String),
    PinAlias(String),
    PinAliasWithArgs(String),
    SelectAndRun,
    SelectAndRunWithArgs,
}

type CmdFn = fn(Array) -> Option<Command>;

impl Command {
    const fn all() -> [(&'static str, CmdFn); NUMBER_CMDS] {
        [
            (CARGO_TOOLS_XTASK_RUN_ALIAS, |arg| {
                try_get_xtask_label(arg).map(Self::RunAlias)
            }),
            (CARGO_TOOLS_XTASK_RUN_ALIAS_WITH_ARGS, |arg| {
                try_get_xtask_label(arg).map(Self::RunAliasWithArgs)
            }),
            (CARGO_TOOLS_XTASK_PIN_ALIAS, |arg| {
                try_get_xtask_label(arg).map(Self::PinAlias)
            }),
            (CARGO_TOOLS_XTASK_PIN_ALIAS_WITH_ARGS, |arg| {
                try_get_xtask_label(arg).map(Self::PinAliasWithArgs)
            }),
            (CARGO_TOOLS_XTASK_SELECT_AND_RUN, |_| {
                Some(Self::SelectAndRun)
            }),
            (CARGO_TOOLS_XTASK_SELECT_AND_RUN_WITH_ARGS, |_| {
                Some(Self::SelectAndRunWithArgs)
            }),
        ]
    }
}

pub fn register_xtask_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_commands(tx, Command::all())
}
