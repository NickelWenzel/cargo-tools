// mod project_outline;

use cargo_tools::app::cargo::{
    command::{Explicit, Implicit},
    selection::{
        Features,
        Update::{self, *},
    },
    ui::Task::*,
};
use iced_headless::Task as IcedTask;
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;

use crate::{
    app::{
        CargoMsg, IntoCargoMessage,
        cargo::{Ui, VsCodeCargoCmd},
    },
    command::SelectInput,
    quick_pick::ToQuickPickItem,
    vs_code_api::{JsValueExt, execute_async, log, show_quick_pick, show_quick_pick_multiple},
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

impl Ui {
    pub(crate) fn process_cmd(&self, cmd: VsCodeCargoCmd) -> IcedTask<CargoMsg> {
        match cmd {
            VsCodeCargoCmd::SelectProfile => {
                let input = self.data.profiles();
                run_task(async move { select(input).await.map(SelectedProfile) })
            }
            VsCodeCargoCmd::SelectPackage => {
                let input = self.data.packages();
                run_task(async move { select(input).await.map(SelectedPackage) })
            }
            VsCodeCargoCmd::SelectBuildTarget => {
                let input = self.data.build_target_options();
                run_task(async move { select(input?).await.map(SelectedBuildTarget) })
            }
            VsCodeCargoCmd::SelectRunTarget => {
                let input = self.data.run_target_options();
                run_task(async move { select(input?).await.map(SelectedRunTarget) })
            }
            VsCodeCargoCmd::SelectBenchmarkTarget => {
                let input = self.data.bench_target_options();
                run_task(async move { select(input?).await.map(SelectedBenchmarkTarget) })
            }
            VsCodeCargoCmd::SelectPlatformTarget => {
                let current = self.data.selection.platform_target.clone();
                run_task(async move { select_platform_target(current.clone()).await })
            }
            VsCodeCargoCmd::InstallPlatformTarget => run_task(install_platform_target()),
            VsCodeCargoCmd::SetRustAnalyzerCheckTargets => {
                IcedTask::done(set_rust_analyzer_check_targets())
                    .and_then(IcedTask::done)
                    .map(IntoCargoMessage::into_cargo_msg)
            }
            VsCodeCargoCmd::BuildDocs => {
                IcedTask::done(ExplicitCommand(Explicit::Doc).into_cargo_msg())
            }
            VsCodeCargoCmd::SelectFeatures => {
                let input = self.data.feature_options();
                run_task(async move { select_features(input).await })
            }
            VsCodeCargoCmd::Refresh => {
                // Not yet implemented
                IcedTask::none()
            }
            VsCodeCargoCmd::Clean => {
                IcedTask::done(ImplicitCommand(Implicit::Clean).into_cargo_msg())
            }
            VsCodeCargoCmd::Build => {
                IcedTask::done(ImplicitCommand(Implicit::Build).into_cargo_msg())
            }
            VsCodeCargoCmd::Run => IcedTask::done(ImplicitCommand(Implicit::Run).into_cargo_msg()),
            VsCodeCargoCmd::Debug => {
                // Not yet implemented
                IcedTask::none()
            }
            VsCodeCargoCmd::Test => {
                IcedTask::done(ImplicitCommand(Implicit::Test).into_cargo_msg())
            }
            VsCodeCargoCmd::Bench => {
                IcedTask::done(ImplicitCommand(Implicit::Bench).into_cargo_msg())
            }
        }
    }
}

fn run_task(
    fut: impl Future<Output = Option<impl IntoCargoMessage + 'static>> + 'static,
) -> IcedTask<CargoMsg> {
    IcedTask::future(fut)
        .and_then(IcedTask::done)
        .map(IntoCargoMessage::into_cargo_msg)
}

async fn select_platform_target(current: Option<String>) -> Option<impl IntoCargoMessage> {
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
            return None;
        }
    };

    let input = {
        let mut options = vec![None];
        options.extend(platform_targets);
        let current = vec![current];
        SelectInput { options, current }
    };

    select(input).await.map(SelectedPlatformTarget)
}

async fn install_platform_target() -> Option<impl IntoCargoMessage> {
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
            return None;
        }
    };

    let input = SelectInput {
        options,
        current: Vec::new(),
    };

    select(input).await.map(AddPlatformTarget)
}

fn set_rust_analyzer_check_targets() -> Option<impl IntoCargoMessage> {
    log("'Set rust-analyzer check targets' not yet implemented");
    Option::<Update>::None
}

async fn select_features(input: Option<SelectInput<String>>) -> Option<impl IntoCargoMessage> {
    let selected_features = select_multiple(input?).await?;
    let features = if selected_features.iter().any(|f| f == "All Features") {
        Features::All
    } else {
        Features::Some(selected_features)
    };

    Some(SelectedFeatures(features))
}
