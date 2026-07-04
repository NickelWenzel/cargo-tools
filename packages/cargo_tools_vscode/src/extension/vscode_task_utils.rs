use std::{collections::HashMap, fmt::Debug};

use futures::{SinkExt, channel::mpsc::Sender};
use iced_viewless::Task;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::quick_pick::show_quick_pick_type;
use tracing::{debug, error};

#[wasm_bindgen(
    raw_module = "../../../packages/cargo_tools_vscode/src/extension/vscode_task_utils.ts"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn register_command(command: &str, callback: &Closure<dyn FnMut(Array)>)
    -> Result<(), JsValue>;
}

pub type CommandBinding = Closure<dyn FnMut(Array)>;
type CommandMap = HashMap<&'static str, CommandBinding>;

pub type OnFileChanged = Closure<dyn FnMut()>;

pub fn send_file_changed(tx: Sender<()>) -> OnFileChanged {
    Closure::new(move || {
        let tx = tx.clone();
        spawn_local(async move {
            if let Err(e) = tx.clone().send(()).await {
                error!("Failed to notify about file change: {e}",)
            }
        })
    })
}

fn register_tasks(cmds: CommandMap) -> Vec<CommandBinding> {
    cmds.into_iter()
        .map(|(command_id, cmd)| {
            debug!("Register task '{command_id}'");
            if let Err(e) = register_command(command_id, &cmd) {
                error!("Failed to register task '{command_id}': {e:?}");
            };
            cmd
        })
        .collect()
}

pub fn take_first<T: DeserializeOwned>(array: Array) -> Option<T> {
    match serde_wasm_bindgen::from_value(array.get(0)) {
        Ok(v) => Some(v),
        Err(e) => {
            error!("Failed to deserialize update: {e}");
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
            error!("Failed to deserialize update: {e_0}, {e_1}");
            None
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Failed to deserialize update: {e}");
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
                error!("Failed to extract command");
                return;
            };
            debug!("Sending VS Code command '{cmd:?}'");
            if let Err(e) = tx.clone().send(cmd).await {
                error!("Failed to queue msg: {}", e);
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

pub fn select_name_filter<C: Clone + Send + 'static>(
    current: String,
    options: Array,
    cmd_tx: Sender<C>,
    make_edit: fn(String) -> C,
) -> Task<C> {
    Task::future(async move {
        let filter_update = Closure::new(move |filter: String| {
            let mut tx = cmd_tx.clone();
            spawn_local(async move {
                if let Err(e) = tx.send(make_edit(filter)).await {
                    error!("Failed to queue filter update: {e}");
                }
            });
        });
        let filter = show_quick_pick_type(current.clone(), options, &filter_update)
            .await
            .map(|f| f.as_string().unwrap_or(current.clone()))
            .unwrap_or(current);
        make_edit(filter)
    })
}
