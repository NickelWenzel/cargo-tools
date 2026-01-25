use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{
        TaskMap, VsCodeTask,
        cargo::command::{CargoCmdFn, Command},
        register_tasks,
    },
    vs_code_api::log,
};

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
                log("Failed to extract cargo command");
                return;
            };
            log(&format!("Sending VS Code cargo command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log(&format!("Failed to queue msg: {}", e));
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
