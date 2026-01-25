use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{
        CargoMakeMsg, TaskMap, VsCodeTask,
        cargo_make::command::{CargoCmdFn, Command},
        register_tasks,
    },
    vs_code_api::log,
};

pub fn register_cargo_make_commands(tx: Sender<Command>) -> Vec<VsCodeTask> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, VsCodeTask);

fn create_vs_code_command(
    tx: Sender<Command>,
    key: &'static str,
    cargo_cmd_fn: CargoCmdFn,
) -> CmdKeyValuePair {
    let cmd = Closure::new(move |args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            let Some(cmd) = cargo_cmd_fn(args) else {
                log("Failed to extract cargo command");
                return;
            };
            log(&format!("Sending VS Code cargo command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    });

    (key, cmd)
}

pub fn task_map(tx: Sender<Command>) -> TaskMap {
    todo!()
    // HashMap::from(
    //     Command::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    // )
}

// fn cargo_make_command_map(tx: Sender<CargoMakeMsg>) -> CommandMap {
//     HashMap::from([
//         (
//             "cargo-tools.makefile.runTask".to_string(),
//             cargo_tools::makefile::run_task(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.selectAndRunTask".to_string(),
//             cargo_tools::makefile::select_and_run_task(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.setTaskFilter".to_string(),
//             cargo_tools::makefile::set_task_filter(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.editTaskFilter".to_string(),
//             cargo_tools::makefile::edit_task_filter(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.clearTaskFilter".to_string(),
//             cargo_tools::makefile::clear_task_filter(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.showCategoryFilter".to_string(),
//             cargo_tools::makefile::show_category_filter(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.clearCategoryFilter".to_string(),
//             cargo_tools::makefile::clear_category_filter(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.add".to_string(),
//             cargo_tools::pinned_makefile_tasks::add(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.remove".to_string(),
//             cargo_tools::pinned_makefile_tasks::remove(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.execute".to_string(),
//             cargo_tools::pinned_makefile_tasks::execute(tx.clone()),
//         ),
//         (
//             "cargo-tools.makefile.pinTask".to_string(),
//             cargo_tools::makefile::pin_task(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.execute1".to_string(),
//             cargo_tools::pinned_makefile_tasks::execute1(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.execute2".to_string(),
//             cargo_tools::pinned_makefile_tasks::execute2(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.execute3".to_string(),
//             cargo_tools::pinned_makefile_tasks::execute3(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.execute4".to_string(),
//             cargo_tools::pinned_makefile_tasks::execute4(tx.clone()),
//         ),
//         (
//             "cargo-tools.pinnedMakefileTasks.execute5".to_string(),
//             cargo_tools::pinned_makefile_tasks::execute5(tx.clone()),
//         ),
//     ])
// }
