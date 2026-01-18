use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{
        Command, CommandMap,
        cargo::{
            CargoToolsCmd::{self},
            command::CargoCmdFn,
        },
        register_commands,
    },
    vs_code_api::log,
};

pub fn register_cargo_commands(tx: Sender<CargoToolsCmd>) -> Vec<Command> {
    register_commands(cargo_command_map(tx))
}

type CmdKeyValuePair = (&'static str, Command);

fn create_vs_code_command(
    tx: Sender<CargoToolsCmd>,
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

pub fn cargo_command_map(tx: Sender<CargoToolsCmd>) -> CommandMap {
    HashMap::from(
        CargoToolsCmd::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    )
}
