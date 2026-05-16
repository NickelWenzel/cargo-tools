use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    commands::pinned::*,
    extension::{CommandBinding, CommandMap, register_tasks},
    vs_code_api::{log_error, log_info, try_get_task_label},
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

impl Command {
    pub const fn all() -> [(&'static str, PinnedCmdFn); NUMBER_CMDS] {
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

type PinnedCmdFn = fn(Array) -> Option<Command>;

pub fn register_pinned_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, CommandBinding);

fn create_vs_code_command(
    tx: Sender<Command>,
    key: &'static str,
    cargo_cmd_fn: PinnedCmdFn,
) -> CmdKeyValuePair {
    let cmd = Closure::new(move |args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            let Some(cmd) = cargo_cmd_fn(args) else {
                log_error("Failed to extract cargo make command");
                return;
            };
            log_info(&format!("Sending VS Code cargo make command '{cmd:?}'"));
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
