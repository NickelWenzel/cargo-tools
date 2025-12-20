use cargo_tools::app::cargo_make::{CargoMakeUi, MakefileTasks};
use cargo_tools::app::state::State;
use std::sync::{Arc, Mutex};
use wasm_async_trait::wasm_async_trait;

pub struct VsCodeCargoMakeUi;

#[wasm_async_trait]
impl CargoMakeUi for VsCodeCargoMakeUi {
    async fn update(_tasks: Arc<Mutex<MakefileTasks>>, _state: Arc<Mutex<State>>) {
        todo!()
    }
}
