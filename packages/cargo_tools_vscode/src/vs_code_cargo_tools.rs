use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{cargo_tools::CargoTools, vs_code_cargo_tools::state_manager::VSCodeStateManager};

mod state_manager;

#[wasm_bindgen]
pub struct VSCodeCargoTools(CargoTools<VSCodeStateManager>);

#[wasm_bindgen]
impl VSCodeCargoTools {
    #[wasm_bindgen]
    pub async fn create() -> Result<Self, JsValue> {
        Ok(Self(CargoTools::create(VSCodeStateManager).await?))
    }
}
