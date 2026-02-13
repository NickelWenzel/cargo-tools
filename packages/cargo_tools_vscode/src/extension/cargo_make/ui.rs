use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::vs_code_api::CargoMakeNode;

/// The data  for the vs code tree item bodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct CargoMakeNodeHandler {}

#[wasm_bindgen]
impl CargoMakeNodeHandler {
    #[wasm_bindgen]
    pub fn tasks(&self) -> Vec<CargoMakeNode> {
        vec![]
    }
}

/// The handler implementing for the vs code tree item bodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct CargoMakeTreeProviderHandler {}

#[wasm_bindgen]
impl CargoMakeTreeProviderHandler {
    #[wasm_bindgen]
    pub fn categories(&self) -> Vec<CargoMakeNode> {
        vec![]
    }
}
