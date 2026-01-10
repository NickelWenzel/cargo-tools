pub mod makefile;
pub mod pinned_makefile_tasks;
pub mod project_outline;
pub mod project_status;

use async_broadcast::Sender;
use cargo_tools::app::cargo::{
    self,
    command::{Explicit, Implicit},
    selection::{self, Features, Update},
    ui::Task,
};
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{self, CargoMsg},
    command::SelectInput,
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

async fn select<T: ToQuickPickItem + Clone + Debug + PartialEq>(
    SelectInput { options, current }: SelectInput<T>,
) -> Option<T> {
    let vccode_options = match options
        .iter()
        .map(|i| {
            let picked = current.contains(i);
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

    let selected_index = match show_quick_pick(vccode_options).await {
        Ok(value) => value.as_f64().map(|f| f as usize),
        Err(e) => {
            log(&format!("Quick pick failed: {e:?}"));
            return None;
        }
    };

    selected_index.and_then(|i| options.get(i)).cloned()
}

async fn select_multiple<T: ToQuickPickItem + Clone + Debug + PartialEq>(
    SelectInput { options, current }: SelectInput<T>,
) -> Option<Vec<T>> {
    let vscode_options = match options
        .iter()
        .map(|i| {
            let picked = current.contains(i);
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

    let selected_indices = match show_quick_pick_multiple(vscode_options).await {
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
            .filter_map(|i| options.get(i).cloned())
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
            if let Some(profile) = select(data.profiles()).await
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
            if let Some(selected) = select(data.packages()).await {
                log(&format!("Sending new package selection: {selected:?}"));
                if let Err(e) = tx_send
                    .broadcast(Update::SelectedPackage(selected).into_cargo_msg())
                    .await
                {
                    log(&format!("Failed to queue msg: {e:?}"));
                }
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
            let Some(input) = data.build_target_options() else {
                log("No build targets found.");
                return;
            };

            if let Some(selected) = select(input).await
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
            let Some(input) = data.run_target_options() else {
                log("No run targets found.");
                return;
            };

            if let Some(selected) = select(input).await
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
            let Some(input) = data.bench_target_options() else {
                log("No benchmark targets found.");
                return;
            };

            if let Some(selected) = select(input).await
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

            let current = vec![data.selection.lock().unwrap().platform_target.clone()];
            let input = SelectInput { options, current };

            if let Some(selected) = select(input).await
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
            let options = match execute_async("rustup target list").await {
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

            let input = SelectInput {
                options,
                current: Vec::new(),
            };

            if let Some(selected) = select(input).await
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
            let Some(input) = data.feature_options() else {
                log("No available features found.");
                return;
            };
            if let Some(selected_features) = select_multiple(input).await {
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
