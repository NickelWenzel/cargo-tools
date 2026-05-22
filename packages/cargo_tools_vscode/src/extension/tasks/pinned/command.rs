use futures::channel::mpsc::Sender;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::pinned::*,
    extension::vscode_task_utils::{CommandBinding, register_commands},
    vs_code_api::try_get_task_label,
};

#[derive(Debug, Clone)]
pub enum Command {
    Add,
    Remove(String),
    Execute(String),
    Execute1,
    Execute2,
    Execute3,
    Execute4,
    Execute5,
}

type CmdFn = fn(Array) -> Option<Command>;

impl Command {
    const fn all() -> [(&'static str, CmdFn); NUMBER_CMDS] {
        [
            (CARGO_TOOLS_PINNED_ADD, |_| Some(Self::Add)),
            (CARGO_TOOLS_PINNED_REMOVE, |arg| {
                try_get_task_label(arg).map(Self::Remove)
            }),
            (CARGO_TOOLS_PINNED_EXECUTE, |arg| {
                try_get_task_label(arg).map(Self::Execute)
            }),
            (CARGO_TOOLS_PINNED_EXECUTE1, |_| Some(Self::Execute1)),
            (CARGO_TOOLS_PINNED_EXECUTE2, |_| Some(Self::Execute2)),
            (CARGO_TOOLS_PINNED_EXECUTE3, |_| Some(Self::Execute3)),
            (CARGO_TOOLS_PINNED_EXECUTE4, |_| Some(Self::Execute4)),
            (CARGO_TOOLS_PINNED_EXECUTE5, |_| Some(Self::Execute5)),
        ]
    }
}

pub fn register_pinned_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_commands(tx, Command::all())
}
