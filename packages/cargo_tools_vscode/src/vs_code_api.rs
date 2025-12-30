use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::JsString;

use crate::runtime::VsCodeTask;

#[wasm_bindgen(raw_module = "../cargoTools.ts")]
extern "C" {
    pub async fn echo_task(msg: &str);

    #[wasm_bindgen(catch)]
    pub async fn execute_async(command: &str) -> Result<JsString, JsValue>;
}

#[wasm_bindgen(raw_module = "../runtime.ts")]
extern "C" {
    /// Start watching the current directory for changes.
    /// Returns a handle that can be used to stop watching.
    pub fn watch_current_dir() -> u32;

    /// Stop watching the current directory.
    pub fn unwatch_current_dir(handle: u32);

    /// Start watching a specific file for changes.
    /// Returns a handle that can be used to stop watching.
    pub fn watch_file(path: &str) -> u32;

    /// Stop watching a specific file.
    pub fn unwatch_file(handle: u32);
}

#[wasm_bindgen(raw_module = "../task.ts")]
extern "C" {
    /// Get a state value from VS Code workspace state storage.
    pub async fn execute_task(task: VsCodeTask);
}

#[wasm_bindgen(raw_module = "../context.ts")]
extern "C" {
    /// Get a state value from VS Code workspace state storage.
    pub fn get_state(key: &str) -> JsValue;

    /// Set a state value in VS Code workspace state storage.
    #[wasm_bindgen(catch)]
    pub async fn set_state(key: &str, value: JsValue) -> Result<(), JsValue>;
}

#[wasm_bindgen(raw_module = "../configuration.ts")]
extern "C" {
    /// Get a configuration value from VS Code settings.
    ///
    /// # Parameters
    /// - `section`: The configuration section (e.g., "cargoTools")
    /// - `key`: The configuration key within the section
    /// - `config_type`: The type of the configuration property
    /// - `default_value`: The default value to return if the key is not found
    pub fn get_config(
        section: &str,
        key: &str,
        config_type: u32,
        default_value: JsValue,
    ) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = ["vscode", "commands"])]
    pub fn execute_command(command: &str, rest: Vec<JsValue>);
}

pub fn set_cargo_context(has_cargo: bool) {
    execute_command(
        "setContext",
        vec![
            JsValue::from_str("cargoTools:workspaceHasCargo"),
            JsValue::from(has_cargo),
        ],
    );
}

pub fn set_makefile_context(has_makefile: bool) {
    execute_command(
        "setContext",
        vec![
            JsValue::from_str("cargoTools:workspaceHasMakefile"),
            JsValue::from(has_makefile),
        ],
    );
}

pub trait JsValueExt {
    fn to_error_string(self) -> String;
}

impl JsValueExt for JsValue {
    fn to_error_string(self) -> String {
        self.as_string().unwrap_or(format!("{self:?}"))
    }
}
