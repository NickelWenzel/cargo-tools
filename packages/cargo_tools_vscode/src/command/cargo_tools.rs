pub mod makefile;
pub mod pinned_makefile_tasks;
pub mod project_outline;
pub mod project_status;

use std::iter;

use async_broadcast::Sender;
use cargo_tools::app::cargo::{
    self,
    selection::{self, Features, Update},
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{self, Message},
    quick_pick::ToQuickPickItem,
    vs_code_api::{JsValueExt, execute_async, log, show_quick_pick, show_quick_pick_multiple},
};

trait IntoMessage {
    fn into_msg(self) -> Message;
}

impl IntoMessage for selection::Update {
    fn into_msg(self) -> Message {
        Message::Cargo(cargo::ui::Message::<_>::Selection(self))
    }
}

async fn select<T: ToQuickPickItem + Clone + PartialEq>(
    items: &[T],
    current_selection: &[T],
) -> Option<T> {
    let items_array = match items
        .iter()
        .map(|i| {
            let picked = current_selection.contains(i);
            to_value(&i.to_item(picked))
        })
        .collect()
    {
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

async fn select_multiple<T: ToQuickPickItem + Clone + PartialEq>(
    items: &[T],
    current_selection: &[T],
) -> Option<Vec<T>> {
    let items_array = match items
        .iter()
        .map(|i| {
            let picked = current_selection.contains(i);
            to_value(&i.to_item(picked))
        })
        .collect()
    {
        Ok(array) => array,
        Err(e) => {
            log(&format!("Failed to serialize quick pick items: {e:?}"));
            return None;
        }
    };

    let selected_indices = match show_quick_pick_multiple(items_array).await {
        Ok(value) => {
            if value.is_null() || value.is_undefined() {
                return None;
            }
            let array =
                wasm_bindgen::JsCast::dyn_ref::<wasm_bindgen_futures::js_sys::Array>(&value)?;
            let indices: Vec<usize> = (0..array.length())
                .filter_map(|i| array.get(i).as_f64().map(|f| f as usize))
                .collect();
            Some(indices)
        }
        Err(e) => {
            log(&format!("Quick pick multiple failed: {e:?}"));
            return None;
        }
    };

    selected_indices.map(|indices| {
        indices
            .into_iter()
            .filter_map(|i| items.get(i).cloned())
            .collect()
    })
}

pub fn select_profile(tx: Sender<Message>, cargo_ui: app::cargo::Ui) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let profiles = ui_clone.data.lock().unwrap().profiles().to_vec();
            let current = ui_clone.selection.lock().unwrap().profile.clone();

            if let Some(profile) = select(&profiles, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedProfile(profile).into_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_package(tx: Sender<Message>, cargo_ui: app::cargo::Ui) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let packages = ui_clone.data.lock().unwrap().package_options();
            let current = ui_clone.selection.lock().unwrap().package.clone();

            if let Some(selected) = select(&packages, &[current]).await {
                if let Err(e) = tx_send
                    .broadcast(Update::SelectedPackage(selected).into_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
}

pub fn select_build_target(
    tx: Sender<Message>,
    cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let targets = ui_clone.build_target_options();
            let current = ui_clone
                .selection
                .lock()
                .unwrap()
                .package_selection()
                .and_then(|s| s.build_target.clone());

            if let Some(selected) = select(&targets, &[current]).await {
                if let Err(e) = tx_send
                    .broadcast(Update::SelectedBuildTarget(selected).into_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
}

pub fn select_run_target(
    tx: Sender<Message>,
    cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let targets = ui_clone.run_target_options();
            let current = ui_clone
                .selection
                .lock()
                .unwrap()
                .package_selection()
                .and_then(|s| s.run_target.clone());

            if let Some(selected) = select(&targets, &[current]).await {
                if let Err(e) = tx_send
                    .broadcast(Update::SelectedRunTarget(selected).into_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
}

pub fn select_benchmark_target(
    tx: Sender<Message>,
    cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let targets = ui_clone.bench_target_options();
            let current = ui_clone
                .selection
                .lock()
                .unwrap()
                .package_selection()
                .and_then(|s| s.benchmark_target.clone());

            if let Some(selected) = select(&targets, &[current]).await {
                if let Err(e) = tx_send
                    .broadcast(Update::SelectedBenchmarkTarget(selected).into_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
}

pub fn select_platform_target(
    tx: Sender<Message>,
    cargo_ui: app::cargo::Ui,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let platform_targets = match execute_async("rustup target list").await {
                Ok(output) => {
                    let output_str = output.as_string().unwrap_or_default();
                    output_str
                        .lines()
                        .map(|line| {
                            let line = line.trim();
                            let installed = line.ends_with("(installed)");
                            if installed {
                                Some(line.trim_end_matches("(installed)").trim().to_string())
                            } else {
                                Some(line.to_string())
                            }
                        })
                        .collect::<Vec<_>>()
                }
                Err(e) => {
                    log(&format!(
                        "Failed to get platform targets from rustup: {}",
                        e.to_error_string()
                    ));
                    vec![]
                }
            };
            let mut options = vec![None];
            options.extend(platform_targets);

            let current = ui_clone.selection.lock().unwrap().platform_target.clone();

            if let Some(selected) = select(&options, &[current]).await {
                if let Err(e) = tx_send
                    .broadcast(Update::SelectedPlatformTarget(selected).into_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
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

pub fn select_features(tx: Sender<Message>, cargo_ui: app::cargo::Ui) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let ui_clone = cargo_ui.clone();
        spawn_local(async move {
            let features = iter::once("All features".to_string())
                .chain(ui_clone.feature_options().into_iter())
                .collect::<Vec<_>>();
            let current_features = match ui_clone.selection.lock().unwrap().selected_features() {
                Features::All => ["All features".to_string()].to_vec(),
                Features::Some(features) => features,
            };

            if let Some(selected_features) = select_multiple(&features, &current_features).await {
                let features = if selected_features.iter().any(|f| f == "All Features") {
                    Features::All
                } else {
                    Features::Some(selected_features)
                };

                if let Err(e) = tx_send
                    .broadcast(Update::SelectedFeatures(features).into_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
}

pub fn refresh(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clean(_tx: Sender<Message>, _cargo_ui: app::cargo::Ui) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
