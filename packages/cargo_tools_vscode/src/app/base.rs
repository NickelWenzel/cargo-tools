use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;

use crate::{
    app::{OnFileChanged, VsCodeTask},
    vs_code_api::{TsFileWatcher, log},
};

#[derive(Debug)]
pub struct Base {
    pub cmds: Vec<VsCodeTask>,
    pub file_watcher: TsFileWatcher,
    pub root_dir: String,
}

pub fn send_file_changed(tx: Sender<()>) -> OnFileChanged {
    Closure::new(move || {
        let tx = tx.clone();
        spawn_local(async move {
            if let Err(e) = tx.clone().send(()).await {
                log(&format!("Failed to notify about file change: {e}",))
            }
        })
    })
}
