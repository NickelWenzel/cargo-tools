use cargo_tools::app::cargo_settings::{CargoSettingsUi, MetadataUpdate};
use cargo_tools::app::state::State;
use std::sync::{Arc, Mutex};
use wasm_async_trait::wasm_async_trait;

pub struct VsCodeCargoSettingsUi;

#[wasm_async_trait]
impl CargoSettingsUi for VsCodeCargoSettingsUi {
    async fn update(_metadata: Arc<Mutex<MetadataUpdate>>, _state: Arc<Mutex<State>>) {
        todo!()
    }
}
