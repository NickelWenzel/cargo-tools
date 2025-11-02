use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test() -> String {
    String::from("This is a test function in the cargo_tools_vscode package.")
}

#[wasm_bindgen]
pub struct CargoTools;

#[wasm_bindgen]
impl CargoTools {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        CargoTools
    }

    pub fn test(&self) -> String {
        test()
    }
}

impl Default for CargoTools {
    fn default() -> Self {
        Self::new()
    }
}
