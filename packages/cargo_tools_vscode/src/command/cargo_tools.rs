pub mod makefile;
pub mod pinned_makefile_tasks;
pub mod project_outline;
pub mod project_status;

use cargo_tools::app::cargo::{
    command::{Explicit, Implicit},
    selection::{Features, Update::*},
    ui::Task,
};
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    command::{CargoCmdData, SelectInput},
    quick_pick::ToQuickPickItem,
    vs_code_api::{
        JsValueExt, execute_async, log, show_quick_pick, show_quick_pick_multiple, showErrorMessage,
    },
};

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
    }?;

    options.get(selected_index).cloned()
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
    }?;

    let selected = selected_indices
        .into_iter()
        .filter_map(|i| options.get(i).cloned())
        .collect();
    Some(selected)
}

pub fn select_profile(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            if let Some(profile) = select(data.profiles()).await
                && let Err(e) = data.send(SelectedProfile(profile)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn select_package(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            if let Some(selected) = select(data.packages()).await
                && let Err(e) = data.send(SelectedPackage(selected)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn select_build_target(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            let Some(input) = data.build_target_options() else {
                log("No build targets found.");
                return;
            };

            if let Some(selected) = select(input).await
                && let Err(e) = data.send(SelectedBuildTarget(selected)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn select_run_target(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            let Some(input) = data.run_target_options() else {
                log("No run targets found.");
                return;
            };

            if let Some(selected) = select(input).await
                && let Err(e) = data.send(SelectedRunTarget(selected)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn select_benchmark_target(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            let Some(input) = data.bench_target_options() else {
                log("No benchmark targets found.");
                return;
            };

            if let Some(selected) = select(input).await
                && let Err(e) = data.send(SelectedBenchmarkTarget(selected)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn select_platform_target(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            let platform_targets = match execute_async("rustup target list").await {
                Ok(output) => {
                    let output_str = output.as_string().unwrap_or_default();
                    output_str
                        .lines()
                        .filter_map(|line| {
                            let line = line.trim();
                            if line.ends_with("(installed)") {
                                Some(line.trim_end_matches("(installed)").trim().to_string())
                            } else {
                                None
                            }
                        })
                        .map(Some)
                        .collect::<Vec<_>>()
                }
                Err(e) => {
                    log(&format!(
                        "Failed to get platform targets from rustup: {}",
                        e.to_error_string()
                    ));
                    return;
                }
            };
            let mut options = vec![None];
            options.extend(platform_targets);

            let current = vec![data.selection.platform_target.clone()];
            let input = SelectInput { options, current };

            if let Some(selected) = select(input).await
                && let Err(e) = data.send(SelectedPlatformTarget(selected)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn install_platform_target(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
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
                    return;
                }
            };

            let input = SelectInput {
                options,
                current: Vec::new(),
            };

            if let Some(selected) = select(input).await
                && let Err(e) = data.send(Task::AddPlatformTarget(selected)).await
            {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn set_rust_analyzer_check_targets(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        showErrorMessage(
            "'Set rust-analyzer check targets' not yet implemented".to_string(),
            Array::new(),
        );
    })
}

pub fn build_docs(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            if let Err(e) = data.send(Task::ExplicitCommand(Explicit::Doc)).await {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}

pub fn select_features(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
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

                if let Err(e) = data.send(SelectedFeatures(features)).await {
                    log(&format!("Failed to queue msg: {}", e));
                }
            }
        });
    })
}

pub fn refresh(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        showErrorMessage("'Refresh' not yet implemented".to_string(), Array::new());
    })
}

pub fn clean(data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {
        let data = data.clone();
        spawn_local(async move {
            let data = data.lock().unwrap();
            if let Err(e) = data.send(Task::ImplicitCommand(Implicit::Clean)).await {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    })
}
