pub mod makefile;
pub mod pinned_makefile_tasks;
pub mod project_outline;
pub mod project_status;

use std::iter;

use async_broadcast::Sender;
use cargo_tools::app::cargo::{
    self,
    command::{Explicit, Implicit},
    selection::{self, Features, Update},
    ui::Task,
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{self, CargoMsg},
    quick_pick::ToQuickPickItem,
    vs_code_api::{
        JsValueExt, execute_async, log, show_quick_pick, show_quick_pick_multiple, showErrorMessage,
    },
};

trait IntoCargoMessage {
    fn into_cargo_msg(self) -> CargoMsg;
}

impl IntoCargoMessage for selection::Update {
    fn into_cargo_msg(self) -> CargoMsg {
        cargo::ui::Message::Selection(self)
    }
}

impl IntoCargoMessage for Task {
    fn into_cargo_msg(self) -> CargoMsg {
        cargo::ui::Message::Task(self)
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

pub fn select_profile(
    tx: Sender<CargoMsg>,
    cmd_data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = cmd_data.clone();
        spawn_local(async move {
            let profiles = data.metadata.lock().unwrap().profiles().to_vec();
            let current = data.selection.lock().unwrap().profile.clone();

            if let Some(profile) = select(&profiles, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedProfile(profile).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_package(
    tx: Sender<CargoMsg>,
    data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = data.clone();
        spawn_local(async move {
            let packages = data.metadata.lock().unwrap().package_options();
            let current = data.selection.lock().unwrap().package.clone();

            if let Some(selected) = select(&packages, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedPackage(selected).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_build_target(
    tx: Sender<CargoMsg>,
    data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = data.clone();
        spawn_local(async move {
            let ui_metadata_guard = data.metadata.lock().unwrap();
            let Some(metadata) = ui_metadata_guard.metadata.as_ref() else {
                log(&format!("No metadata to select from."));
                return;
            };

            let selection_guard = data.selection.lock().unwrap();

            let targets = selection_guard.build_target_options(metadata);
            let current = selection_guard
                .package_selection()
                .and_then(|s| s.build_target.clone());

            if let Some(selected) = select(&targets, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedBuildTarget(selected).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_run_target(
    tx: Sender<CargoMsg>,
    data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = data.clone();
        spawn_local(async move {
            let ui_metadata_guard = data.metadata.lock().unwrap();
            let Some(metadata) = ui_metadata_guard.metadata.as_ref() else {
                log(&format!("No metadata to select from."));
                return;
            };

            let selection_guard = data.selection.lock().unwrap();

            let targets = selection_guard.run_target_options(metadata);
            let current = selection_guard
                .package_selection()
                .and_then(|s| s.run_target.clone());

            if let Some(selected) = select(&targets, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedRunTarget(selected).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_benchmark_target(
    tx: Sender<CargoMsg>,
    data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = data.clone();
        spawn_local(async move {
            let ui_metadata_guard = data.metadata.lock().unwrap();
            let Some(metadata) = ui_metadata_guard.metadata.as_ref() else {
                log(&format!("No metadata to select from."));
                return;
            };

            let selection_guard = data.selection.lock().unwrap();

            let targets = selection_guard.bench_target_options(metadata);
            let current = selection_guard
                .package_selection()
                .and_then(|s| s.benchmark_target.clone());

            if let Some(selected) = select(&targets, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedBenchmarkTarget(selected).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_platform_target(
    tx: Sender<CargoMsg>,
    data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = data.clone();
        spawn_local(async move {
            let platform_targets = match execute_async("rustup target list").await {
                Ok(output) => {
                    let output_str = output.as_string().unwrap_or_default();
                    output_str
                        .lines()
                        .filter_map(|line| {
                            let line = line.trim();
                            if line.ends_with("(installed)") {
                                Some(Some(
                                    line.trim_end_matches("(installed)").trim().to_string(),
                                ))
                            } else {
                                None
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

            let current = data.selection.lock().unwrap().platform_target.clone();

            if let Some(selected) = select(&options, &[current]).await
                && let Err(e) = tx_send
                    .broadcast(Update::SelectedPlatformTarget(selected).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn install_platform_target(tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        spawn_local(async move {
            let platform_targets = match execute_async("rustup target list").await {
                Ok(output) => {
                    let output_str = output.as_string().unwrap_or_default();
                    output_str
                        .lines()
                        .filter_map(|line| {
                            let line = line.trim();
                            if line.ends_with("(installed)") {
                                None
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

            if let Some(selected) = select(&platform_targets, &[]).await
                && let Err(e) = tx_send
                    .broadcast(Task::AddPlatformTarget(selected).into_cargo_msg())
                    .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn set_rust_analyzer_check_targets(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        showErrorMessage(
            "'Set rust-analyzer check targets' not yet implemented".to_string(),
            Array::new(),
        );
    })
}

pub fn build_docs(tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        spawn_local(async move {
            if let Err(e) = tx_send
                .broadcast(Task::ExplicitCommand(Explicit::Doc).into_cargo_msg())
                .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}

pub fn select_features(
    tx: Sender<CargoMsg>,
    cmd_data: app::cargo::CommandData,
) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        let data = cmd_data.clone();
        spawn_local(async move {
            let ui_metadata_guard = data.metadata.lock().unwrap();
            let Some(metadata) = ui_metadata_guard.metadata.as_ref() else {
                log(&format!("No metadata to select from."));
                return;
            };

            let selection_guard = data.selection.lock().unwrap();
            let features = iter::once("All features".to_string())
                .chain(selection_guard.feature_options(metadata))
                .collect::<Vec<_>>();
            let current_features = match data.selection.lock().unwrap().selected_features() {
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
                    .broadcast(Update::SelectedFeatures(features).into_cargo_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
            }
        });
    })
}

pub fn refresh(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        showErrorMessage("'Refresh' not yet implemented".to_string(), Array::new());
    })
}

pub fn clean(tx: Sender<CargoMsg>, _data: app::cargo::CommandData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let tx_send = tx.clone();
        spawn_local(async move {
            if let Err(e) = tx_send
                .broadcast(Task::ImplicitCommand(Implicit::Clean).into_cargo_msg())
                .await
            {
                log(&format!("Failed to queue msg: {e:?}"));
            }
        });
    })
}
