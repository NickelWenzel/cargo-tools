use cargo_tools::app::cargo::{command::Implicit, ui::Task};
use wasm_async_trait::wasm_async_trait;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::CargoMsgTx,
    command::CargoCmdData,
    vs_code_api::{log, showErrorMessage},
};

#[wasm_async_trait]
pub trait SendImplicitCmd {
    async fn send_cmd(&self, cmd: Implicit);
}

#[wasm_async_trait]
impl<T: CargoMsgTx> SendImplicitCmd for T {
    async fn send_cmd(&self, cmd: Implicit) {
        if let Err(e) = CargoMsgTx::send(self, Task::ImplicitCommand(cmd)).await {
            log(&format!("Failed to queue msg: {}", e));
        }
    }
}

pub fn build(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let tx = data.lock().unwrap().tx.clone();
            tx.send_cmd(Implicit::Build).await;
        });
    })
}

pub fn run(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let tx = data.lock().unwrap().tx.clone();
            tx.send_cmd(Implicit::Run).await;
        });
    })
}

pub fn debug(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        showErrorMessage("'Debug' not yet implemented".to_string(), Array::new());
    })
}

pub fn test(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let tx = data.lock().unwrap().tx.clone();
            tx.send_cmd(Implicit::Test).await;
        });
    })
}

pub fn bench(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let tx = data.lock().unwrap().tx.clone();
            tx.send_cmd(Implicit::Bench).await;
        });
    })
}
