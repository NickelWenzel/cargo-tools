use futures::channel::mpsc::Sender;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::xtask::*,
    extension::vscode_task_utils::{CommandBinding, register_commands},
    vs_code_api::try_get_xtask_label,
};

#[derive(Debug, Clone)]
pub enum Command {
    RunAlias(String),
    SelectAndRun,
    SelectFilter,
    EditFilter(String),
    ClearFilter,
}

type CmdFn = fn(Array) -> Option<Command>;

impl Command {
    const fn all() -> [(&'static str, CmdFn); NUMBER_CMDS] {
        [
            (CARGO_TOOLS_XTASK_RUN_ALIAS, |arg| {
                try_get_xtask_label(arg).map(Self::RunAlias)
            }),
            (CARGO_TOOLS_XTASK_SELECT_AND_RUN, |_| {
                Some(Self::SelectAndRun)
            }),
            (CARGO_TOOLS_XTASK_SELECT_FILTER, |_| {
                Some(Self::SelectFilter)
            }),
            (CARGO_TOOLS_XTASK_CLEAR_FILTER, |_| Some(Self::ClearFilter)),
        ]
    }
}

pub fn register_xtask_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_commands(tx, Command::all())
}
