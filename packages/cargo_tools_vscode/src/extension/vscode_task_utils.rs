use std::{collections::HashMap, fmt::Debug};

use futures::{SinkExt, channel::mpsc::Sender};
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::vs_code_api::{log_error, log_info, register_command};

pub type CommandBinding = Closure<dyn FnMut(Array)>;
type CommandMap = HashMap<&'static str, CommandBinding>;

pub type OnFileChanged = Closure<dyn FnMut()>;

pub fn send_file_changed(tx: Sender<()>) -> OnFileChanged {
    Closure::new(move || {
        let tx = tx.clone();
        spawn_local(async move {
            if let Err(e) = tx.clone().send(()).await {
                log_error(&format!("Failed to notify about file change: {e}",))
            }
        })
    })
}

fn register_tasks(cmds: CommandMap) -> Vec<CommandBinding> {
    cmds.into_iter()
        .map(|(command_id, cmd)| {
            log_info(&format!("Register task '{command_id}'"));
            if let Err(e) = register_command(command_id, &cmd) {
                log_error(&format!("Failed to register task '{command_id}': {e:?}"));
            };
            cmd
        })
        .collect()
}

pub fn take_first<T: DeserializeOwned>(array: Array) -> Option<T> {
    match serde_wasm_bindgen::from_value(array.get(0)) {
        Ok(v) => Some(v),
        Err(e) => {
            log_error(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}

pub fn take_first_two<T: DeserializeOwned, U: DeserializeOwned>(array: Array) -> Option<(T, U)> {
    match (
        serde_wasm_bindgen::from_value(array.get(0)),
        serde_wasm_bindgen::from_value(array.get(1)),
    ) {
        (Ok(v_0), Ok(v_1)) => Some((v_0, v_1)),
        (Err(e_0), Err(e_1)) => {
            log_error(&format!("Failed to deserialize update: {e_0}, {e_1}"));
            None
        }
        (Err(e), _) | (_, Err(e)) => {
            log_error(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}

fn create_vs_code_command<Cmd: Debug + 'static>(
    tx: Sender<Cmd>,
    key: &'static str,
    cmd_fn: fn(Array) -> Option<Cmd>,
) -> (&'static str, CommandBinding) {
    let cmd = Closure::new(move |args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            let Some(cmd) = cmd_fn(args) else {
                log_error("Failed to extract command");
                return;
            };
            log_info(&format!("Sending VS Code command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log_error(&format!("Failed to queue msg: {}", e));
            }
        });
    });
    (key, cmd)
}

type CmdArray<Cmd, const N: usize> = [(&'static str, fn(Array) -> Option<Cmd>); N];

pub fn register_commands<Cmd: Debug + 'static, const N: usize>(
    tx: Sender<Cmd>,
    all: CmdArray<Cmd, N>,
) -> Vec<CommandBinding> {
    register_tasks(HashMap::from(
        all.map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    ))
}
