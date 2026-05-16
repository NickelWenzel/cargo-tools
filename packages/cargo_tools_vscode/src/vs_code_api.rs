use std::{fmt::Debug, ops::Deref};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::{Array, JsString};

use crate::{
    extension::{
        OnFileChanged,
        cargo::ui::{
            CargoConfigurationTreeProviderHandler, CargoOutlineTreeProviderHandler, NodeType,
            OutlineNodeType,
        },
        cargo_make::ui::{
            CargoMakeNodeHandler, CargoMakePinnedTreeProviderHandler, CargoMakeTreeProviderHandler,
        },
    },
    icon::Icon,
    runtime::{VsCodeProcess, VsCodeTask},
};

#[wasm_bindgen(raw_module = "../execute.ts")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn execute_async(process: VsCodeProcess) -> Result<JsString, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn executeCommand(command: &str, rest: Array) -> Result<JsValue, JsValue>;

    /// Get a state value from VS Code workspace state storage.
    pub async fn execute_task(task: VsCodeTask);

    #[wasm_bindgen(catch)]
    pub async fn showInformationMessage(
        message: String,
        items: Vec<String>,
    ) -> Result<JsString, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn showErrorMessage(message: String, items: Vec<String>)
    -> Result<JsString, JsValue>;
}

#[wasm_bindgen(raw_module = "../runtime.ts")]
extern "C" {
    pub type FileWatcher;

    #[wasm_bindgen(constructor)]
    pub fn new() -> FileWatcher;

    #[wasm_bindgen(method)]
    fn on_changed(this: &FileWatcher, callback: &OnFileChanged);

    #[wasm_bindgen(method)]
    pub fn watch_files(this: &FileWatcher, paths: Vec<String>);

    #[wasm_bindgen(catch)]
    pub async fn read_file(file_path: &str) -> Result<JsString, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn debug(target_exe_path: &str, target_name: &str) -> Result<JsValue, JsValue>;

    pub fn host_platform() -> String;

    pub fn log_debug(msg: &str);
    pub fn log_info(msg: &str);
    pub fn log_warn(msg: &str);
    pub fn log_error(msg: &str);

    /// Get a state value from VS Code workspace state storage.
    #[wasm_bindgen(catch)]
    pub fn register_command(
        command: &str,
        callback: &Closure<dyn FnMut(Array)>,
    ) -> Result<(), JsValue>;

    /// Get a state value from VS Code workspace state storage.
    #[wasm_bindgen(catch)]
    pub fn get_state(key: &str) -> Result<String, JsValue>;

    /// Set a state value in VS Code workspace state storage.
    #[wasm_bindgen(catch)]
    pub async fn set_state(key: &str, value: String) -> Result<(), JsValue>;
}

pub struct TsFileWatcher {
    file_watcher: FileWatcher,
    _on_file_changed: OnFileChanged,
}

impl TsFileWatcher {
    pub fn new(callback: OnFileChanged) -> Self {
        let file_watcher = FileWatcher::new();
        file_watcher.on_changed(&callback);
        Self {
            file_watcher,
            _on_file_changed: callback,
        }
    }
}

impl Deref for TsFileWatcher {
    type Target = FileWatcher;

    fn deref(&self) -> &Self::Target {
        &self.file_watcher
    }
}

impl Debug for TsFileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TsFileWatcher").finish()
    }
}

#[wasm_bindgen(raw_module = "../configuration.ts")]
extern "C" {
    /// Get a configuration value from VS Code settings.
    ///
    /// # Parameters
    /// - `section`: The configuration section (e.g., "cargoTools")
    /// - `key`: The configuration key within the section
    /// - `config_value_type`: The type of the value retrieved
    /// - `default_value`: The default value to return if the key is not found
    pub fn get_config(
        section: &str,
        key: &str,
        config_value_type: u32,
        default_value: JsValue,
    ) -> JsValue;

    /// Get rust analyzer check targets from VS Code settings.
    pub fn get_rust_analyzer_check_targets() -> Vec<String>;

    /// Get a configuration value from VS Code settings.
    ///
    /// # Parameters
    /// - `targets`: The new rust analyzer check targets
    pub async fn update_rust_analyzer_check_targets(targets: Vec<String>);
}

#[wasm_bindgen(raw_module = "../window.ts")]
extern "C" {
    /// Show a quick pick menu to the user.
    ///
    /// # Parameters
    /// - `items`: Array of quick pick items to display
    ///
    /// # Returns
    /// - `Ok(JsValue)`: Index of the selected item (as a number), or null if cancelled
    /// - `Err(JsValue)`: Error if the operation fails
    #[wasm_bindgen(catch)]
    pub async fn show_quick_pick(items: Array) -> Result<JsValue, JsValue>;

    /// Show a multi-select quick pick menu to the user.
    ///
    /// # Parameters
    /// - `items`: Array of quick pick items to display
    ///
    /// # Returns
    /// - `Ok(JsValue)`: Array of selected indices, or null if cancelled
    /// - `Err(JsValue)`: Error if the operation fails
    #[wasm_bindgen(catch)]
    pub async fn show_quick_pick_multiple(
        items: Array,
        on_pick: &Closure<dyn FnMut(Vec<String>)>,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn show_quick_pick_type(
        current: String,
        items: Array,
        callback: &Closure<dyn FnMut(String)>,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(raw_module = "../cargoMakeTreeProvider.ts")]
extern "C" {
    pub type CargoMakeNode;

    #[wasm_bindgen(constructor)]
    pub fn new(
        label: String,
        icon: Icon,
        collapsible_state: u32,
        context_value: String,
        description: String,
        handler: CargoMakeNodeHandler,
        tooltip: Option<String>,
    ) -> CargoMakeNode;

    pub type CargoMakeTreeProvider;

    #[wasm_bindgen(constructor)]
    pub fn new(handler: CargoMakeTreeProviderHandler) -> CargoMakeTreeProvider;

    #[wasm_bindgen(method)]
    pub fn update(this: &CargoMakeTreeProvider, handler: CargoMakeTreeProviderHandler);

    pub type CargoMakePinnedNode;

    #[wasm_bindgen(constructor)]
    pub fn new(
        label: String,
        icon: Icon,
        collapsible_state: u32,
        context_value: String,
        description: String,
        tooltip: String,
        handler: CargoMakeNodeHandler,
    ) -> CargoMakePinnedNode;

    #[wasm_bindgen]
    pub fn try_get_task_label(value: Array) -> Option<String>;

    pub type CargoMakePinnedTreeProvider;

    #[wasm_bindgen(constructor)]
    pub fn new(handler: CargoMakePinnedTreeProviderHandler) -> CargoMakePinnedTreeProvider;

    #[wasm_bindgen(method)]
    pub fn update(this: &CargoMakePinnedTreeProvider, handler: CargoMakePinnedTreeProviderHandler);
}

impl Debug for CargoMakeTreeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CargoMakeTreeProvider").finish()
    }
}

impl Debug for CargoMakePinnedTreeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CargoMakePinnedTreeProvider").finish()
    }
}

#[wasm_bindgen(raw_module = "../configurationTreeProvider.ts")]
extern "C" {
    pub type CargoNode;

    #[wasm_bindgen(constructor)]
    pub fn new(
        label: String,
        icon: Icon,
        collapsible_state: u32,
        handler: NodeType,
        context_value: Option<String>,
        description: Option<String>,
        tooltip: Option<String>,
        command: Option<String>,
        command_arg: Option<String>,
    ) -> CargoNode;

    pub type CargoConfigurationTreeProvider;

    #[wasm_bindgen(constructor)]
    pub fn new(handler: CargoConfigurationTreeProviderHandler) -> CargoConfigurationTreeProvider;

    #[wasm_bindgen(method)]
    pub fn update(this: &CargoConfigurationTreeProvider);
}

impl Debug for CargoConfigurationTreeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CargoConfigurationTreeProvider").finish()
    }
}

#[wasm_bindgen(raw_module = "../outlineTreeProvider.ts")]
extern "C" {
    pub type CargoOutlineNode;

    #[wasm_bindgen(constructor)]
    pub fn new(
        label: String,
        icon: Icon,
        collapsible_state: u32,
        node_type: OutlineNodeType,
        context_value: Option<String>,
        description: Option<String>,
        tooltip: Option<String>,
        command: Option<String>,
        command_args: Option<String>,
    ) -> CargoOutlineNode;

    #[wasm_bindgen(static_method_of = CargoOutlineNode)]
    pub fn feature(
        label: String,
        icon: Icon,
        collapsible_state: u32,
        node_type: OutlineNodeType,
        command: String,
        command_args: Vec<String>,
    ) -> CargoOutlineNode;

    pub type CargoOutlineTreeProvider;

    #[wasm_bindgen(constructor)]
    pub fn new(handler: CargoOutlineTreeProviderHandler) -> CargoOutlineTreeProvider;

    #[wasm_bindgen(method)]
    pub fn update(this: &CargoOutlineTreeProvider);

    #[wasm_bindgen]
    pub fn try_get_node_type(value: Array) -> Option<OutlineNodeType>;
}

impl Debug for CargoOutlineTreeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CargoOutlineTreeProvider").finish()
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
        log_error(&e.to_error_string());
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
        log_error(&e.to_error_string());
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
