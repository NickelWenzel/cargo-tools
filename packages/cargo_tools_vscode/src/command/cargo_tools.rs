pub mod makefile;
pub mod pinned_makefile_tasks;
pub mod project_outline;
pub mod project_status;

use std::sync::{Arc, Mutex};

use async_broadcast::Sender;
use cargo_tools::app::cargo::{
    self,
    selection::{self, Update},
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{self, Message},
    quick_pick::ToQuickPickItem,
    vs_code_api::{log, show_quick_pick},
};

trait IntoMessage {
    fn into_msg(self) -> Message;
}

impl IntoMessage for selection::Update {
    fn into_msg(self) -> Message {
        Message::Cargo(cargo::ui::Message::<_>::Selection(self))
    }
}

async fn select<T: ToQuickPickItem + Clone>(items: &[T]) -> Option<T> {
    let items_array = match items.iter().map(T::to_item).map(|i| to_value(&i)).collect() {
        Ok(array) => array,
        Err(e) => {
            log(&format!("Failed to serialize quick pick items: {e:?}"));
            return None;
        }
    };

    let selected_index = match show_quick_pick(items_array).await {
        Ok(value) => value.as_f64().map(|f| f as usize),
        Err(e) => {
            log(&format!("Quick pick failed: {e:?}"));
            return None;
        }
    };

    selected_index.and_then(|i| items.get(i)).cloned()
}

pub fn select_profile(
    tx: Sender<Message>,
    data: Arc<Mutex<app::cargo::Data>>,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data_clone = data.clone();
        spawn_local(async move {
            let profiles = data_clone.lock().unwrap().profiles().to_vec();

            if let Some(profile) = select(&profiles).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedProfile(profile).into_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_package(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn select_build_target(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn select_run_target(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn select_benchmark_target(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn select_platform_target(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn install_platform_target(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_rust_analyzer_check_targets(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_docs(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn select_features(
    _tx: Sender<Message>,
    _cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn refresh(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clean(_tx: Sender<Message>, _cargo_ui: app::cargo::Ui) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
