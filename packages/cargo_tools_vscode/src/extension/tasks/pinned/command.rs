use futures::channel::mpsc::Sender;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::pinned::*,
    extension::{
        tasks::cargo_make::tree_provider::try_get_task_label,
        vscode_task_utils::{CommandBinding, register_commands},
    },
};

#[wasm_bindgen(raw_module = "../cargoMakeTreeProvider.ts")]
extern "C" {
    #[wasm_bindgen]
    fn try_get_pinned_alias_key(value: Array) -> Option<String>;
}

#[derive(Debug, Clone)]
pub enum Command {
    Add,
    Remove(String),
    Execute(String),
    ExecuteAlias(String),
    RemoveAlias(String),
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
            (CARGO_TOOLS_PINNED_EXECUTE_ALIAS, |arg| {
                try_get_pinned_alias_key(arg).map(Self::ExecuteAlias)
            }),
            (CARGO_TOOLS_PINNED_REMOVE_ALIAS, |arg| {
                try_get_pinned_alias_key(arg).map(Self::RemoveAlias)
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
