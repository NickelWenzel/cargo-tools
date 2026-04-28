use cargo_tools::task::{CargoTask, Task};
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Map;

use crate::vs_code_api::*;

pub const CHANNEL_CAPACITY: usize = 100;

pub async fn read_file_vs_code(file_path: String) -> Result<String, String> {
    read_file(&file_path)
        .await
        .map(|js_str| js_str.as_string().expect("JsString conversion failed"))
        .map_err(|e| e.to_error_string())
}

pub async fn persist_state_vs_code(key: String, state: impl Serialize) {
    let state = serde_json::to_string(&state);
    let Ok(state) = state else {
        let e = state.unwrap_err();
        log_error(&format!("Failed to serialize state: {e}"));
        return;
    };

    if let Err(e) = set_state(&key, state).await {
        let e = e.to_error_string();
        log_error(&format!("Failed to set state: {e}"));
    }
}

pub fn get_state_vs_code<T: DeserializeOwned + Debug>(key: String) -> Option<T> {
    let state = get_state(&key);
    let Ok(state) = state else {
        log_info(&format!(
            "Failed to get state: {}",
            state.unwrap_err().to_error_string()
        ));
        return None;
    };
    let state = serde_json::from_str(&state);
    let Ok(state) = state else {
        let e = state.unwrap_err();
        log_error(&format!("Failed to deserialize state: {e}"));
        return None;
    };
    Some(state)
}

pub async fn exec_vs_code(command: String, args: Vec<String>) -> Result<String, String> {
    execute_async(&command, args)
        .await
        .map(|js_str| js_str.as_string().expect("JsString conversion failed"))
        .map_err(|e| e.to_error_string())
}

pub async fn exec_task_vs_code(task: CargoTask) {
    execute_task(VsCodeTask(task)).await;
}

/// Task type which is exported in typescript code
#[wasm_bindgen]
pub struct VsCodeTask(CargoTask);

#[wasm_bindgen]
impl VsCodeTask {
    #[wasm_bindgen]
    pub fn task_type(&self) -> String {
        match self.0 {
            CargoTask::Cargo(_) => "cargo-tools-cargo".to_string(),
            CargoTask::CargoMake(_) => "cargo-tools-cargo-make".to_string(),
            CargoTask::RustUp(_) => "cargo-tools-cargo".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn cmd(&self) -> String {
        self.task().cmd.clone()
    }

    #[wasm_bindgen]
    pub fn args(&self) -> Vec<String> {
        self.task().args.clone()
    }

    #[wasm_bindgen]
    pub fn env(&self) -> Map {
        to_value(&self.task().env)
            .map(Map::from)
            .unwrap_or_default()
    }

    fn task(&self) -> &Task {
        match &self.0 {
            CargoTask::Cargo(task) => task,
            CargoTask::CargoMake(task) => task,
            CargoTask::RustUp(task) => task,
        }
    }
}
