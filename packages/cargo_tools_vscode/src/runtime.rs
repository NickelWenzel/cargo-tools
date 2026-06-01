use cargo_tools::process::Process;
use serde::{Serialize, de::DeserializeOwned};
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;
use tracing::{error, info};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::{Array, JsString, Map};

#[wasm_bindgen(raw_module = "../../../packages/cargo_tools_vscode/src/runtime.ts")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn execute_async(process: VsCodeProcess) -> Result<JsString, JsValue>;

    pub async fn execute_task(task: VsCodeTask);

    #[wasm_bindgen(catch)]
    async fn executeCommand(command: &str, rest: Array) -> Result<JsValue, JsValue>;

    type FileWatcher;

    #[wasm_bindgen(constructor)]
    fn new() -> FileWatcher;

    #[wasm_bindgen(method)]
    fn on_changed(this: &FileWatcher, callback: &Closure<dyn FnMut()>);

    #[wasm_bindgen(method)]
    fn watch_files(this: &FileWatcher, paths: Vec<String>);

    #[wasm_bindgen(catch)]
    async fn read_file(file_path: &str) -> Result<JsString, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn debug(target_exe_path: &str, target_name: &str) -> Result<JsValue, JsValue>;

    pub fn host_platform() -> String;

    #[wasm_bindgen(catch)]
    fn get_state(key: &str) -> Result<String, JsValue>;

    #[wasm_bindgen(catch)]
    async fn set_state(key: &str, value: String) -> Result<(), JsValue>;
}

pub struct TsFileWatcher {
    file_watcher: FileWatcher,
    _on_file_changed: Closure<dyn FnMut()>,
}

impl TsFileWatcher {
    pub fn new(callback: Closure<dyn FnMut()>) -> Self {
        let file_watcher = FileWatcher::new();
        file_watcher.on_changed(&callback);
        Self {
            file_watcher,
            _on_file_changed: callback,
        }
    }

    pub fn watch_files(&self, paths: Vec<String>) {
        self.file_watcher.watch_files(paths);
    }
}

impl Debug for TsFileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TsFileWatcher").finish()
    }
}

pub async fn set_cargo_context(has_cargo: bool) {
    let res = executeCommand(
        "setContext",
        Array::of2(
            &JsValue::from_str("cargoTools:workspaceHasCargo"),
            &JsValue::from_bool(has_cargo),
        ),
    )
    .await;
    if let Err(e) = res {
        error!("{}", e.to_error_string());
    }
}

pub async fn set_xtask_context(has_xtask_config: bool) {
    let res = executeCommand(
        "setContext",
        Array::of2(
            &JsValue::from_str("cargoTools:workspaceHasXtaskConfig"),
            &JsValue::from_bool(has_xtask_config),
        ),
    )
    .await;
    if let Err(e) = res {
        error!("{}", e.to_error_string());
    }
}

pub async fn set_makefile_context(has_makefile: bool) {
    let res = executeCommand(
        "setContext",
        Array::of2(
            &JsValue::from_str("cargoTools:workspaceHasMakefile"),
            &JsValue::from_bool(has_makefile),
        ),
    )
    .await;
    if let Err(e) = res {
        error!("{}", e.to_error_string());
    }
}

pub trait JsValueExt {
    fn to_error_string(self) -> String;
}

impl JsValueExt for JsValue {
    fn to_error_string(self) -> String {
        self.as_string().unwrap_or(format!("{self:?}"))
    }
}

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
        error!("Failed to serialize state: {e}");
        return;
    };

    if let Err(e) = set_state(&key, state).await {
        let e = e.to_error_string();
        error!("Failed to set state: {e}");
    }
}

pub fn get_state_vs_code<T: DeserializeOwned + Debug>(key: String) -> Option<T> {
    let state = get_state(&key);
    let Ok(state) = state else {
        info!(
            "Failed to get state: {}",
            state.unwrap_err().to_error_string()
        );
        return None;
    };
    let state = serde_json::from_str(&state);
    let Ok(state) = state else {
        let e = state.unwrap_err();
        error!("Failed to deserialize state: {e}");
        return None;
    };
    Some(state)
}

pub async fn exec_vs_code(process: Process) -> Result<String, String> {
    execute_async(VsCodeProcess(process))
        .await
        .map(|js_str| js_str.as_string().expect("JsString conversion failed"))
        .map_err(|e| e.to_error_string())
}

trait ProcessExt {
    fn js_env(&self) -> Map;
}

impl ProcessExt for Process {
    fn js_env(&self) -> Map {
        to_value(&self.env()).map(Map::from).unwrap_or_default()
    }
}

/// Task type which is exported in typescript code
#[wasm_bindgen]
pub struct VsCodeProcess(Process);

#[wasm_bindgen]
impl VsCodeProcess {
    #[wasm_bindgen]
    pub fn cmd(&self) -> String {
        self.0.cmd().to_string()
    }

    #[wasm_bindgen]
    pub fn args(&self) -> Vec<String> {
        self.0.args().to_vec()
    }

    #[wasm_bindgen]
    pub fn env(&self) -> Map {
        self.0.js_env()
    }
}

/// Gives the context in which a [Task] is run
enum CargoTask {
    Cargo(Process),
    CargoMake(Process),
    RustUp(Process),
    XtaskAlias(Process),
}

/// Task type which is exported in typescript code
#[wasm_bindgen]
pub struct VsCodeTask(CargoTask);

impl VsCodeTask {
    pub fn cargo(process: Process) -> Self {
        Self(CargoTask::Cargo(process))
    }

    pub fn cargo_make(process: Process) -> Self {
        Self(CargoTask::CargoMake(process))
    }

    pub fn rustup(process: Process) -> Self {
        Self(CargoTask::RustUp(process))
    }

    pub fn xtask_alias(process: Process) -> Self {
        Self(CargoTask::XtaskAlias(process))
    }

    fn process(&self) -> &Process {
        match &self.0 {
            CargoTask::Cargo(process) => process,
            CargoTask::CargoMake(process) => process,
            CargoTask::RustUp(process) => process,
            CargoTask::XtaskAlias(process) => process,
        }
    }
}

#[wasm_bindgen]
impl VsCodeTask {
    #[wasm_bindgen]
    pub fn task_type(&self) -> String {
        match self.0 {
            CargoTask::Cargo(_) => "cargo-tools-cargo".to_string(),
            CargoTask::CargoMake(_) => "cargo-tools-cargo-make".to_string(),
            CargoTask::RustUp(_) => "cargo-tools-cargo".to_string(),
            CargoTask::XtaskAlias(_) => "cargo-tools-xtask".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn cmd(&self) -> String {
        self.process().cmd().to_string()
    }

    #[wasm_bindgen]
    pub fn args(&self) -> Vec<String> {
        self.process().args().to_vec()
    }

    #[wasm_bindgen]
    pub fn env(&self) -> Map {
        self.process().js_env()
    }
}
