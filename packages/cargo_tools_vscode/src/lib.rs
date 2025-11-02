use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../cargoTools.ts")]
extern "C" {
    async fn echo_task(msg: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct CargoTools;

#[wasm_bindgen]
impl CargoTools {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        log("Creating new CargoTools instance");
        CargoTools
    }

    pub async fn test(&self) {
        echo_task("Test echo task from CargoTools").await;
    }
}

impl Default for CargoTools {
    fn default() -> Self {
        Self::new()
    }
}
