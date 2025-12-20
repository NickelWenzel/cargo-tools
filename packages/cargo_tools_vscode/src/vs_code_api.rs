use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::JsString;

#[wasm_bindgen(raw_module = "../cargoTools.ts")]
extern "C" {
    pub async fn echo_task(msg: &str);

    #[wasm_bindgen(catch)]
    pub async fn execute_async(command: &str) -> Result<JsString, JsValue>;
}

#[wasm_bindgen(raw_module = "../stateManager.ts")]
extern "C" {
    #[wasm_bindgen]
    pub fn get_state(key: &str) -> Option<JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn update_state(key: String, value: JsValue) -> Result<(), JsValue>;
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
