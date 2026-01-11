use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    app::{self, CargoMakeMsg, cargo_make},
    quick_pick::ToQuickPickItem,
    vs_code_api::{log, register_command},
};

mod cargo_tools;

type CargoCmdData = Arc<Mutex<app::cargo::CommandData>>;

pub fn register_cargo_commands(data: CargoCmdData) -> Vec<Command> {
    cargo_command_map(data)
        .into_iter()
        .map(|(command_id, cmd)| {
            if let Err(e) = register_command(&command_id, &cmd) {
                log(&format!(
                    "Failed to register command '{}': {:?}",
                    command_id, e
                ));
            };
            cmd
        })
        .collect()
}

pub fn register_cargo_make_commands(
    tx: Sender<CargoMakeMsg>,
    data: cargo_make::CommandData,
) -> Vec<Command> {
    cargo_make_command_map(tx, data)
        .into_iter()
        .map(|(command_id, cmd)| {
            if let Err(e) = register_command(&command_id, &cmd) {
                log(&format!(
                    "Failed to register command '{}': {:?}",
                    command_id, e
                ));
            };
            cmd
        })
        .collect()
}

pub type Command = Closure<dyn FnMut(Array)>;
type CommandMap = HashMap<String, Closure<dyn FnMut(Array)>>;

fn cargo_command_map(data: CargoCmdData) -> CommandMap {
    HashMap::from([
        (
            "cargo-tools.selectProfile".to_string(),
            cargo_tools::select_profile(data.clone()),
        ),
        (
            "cargo-tools.selectPackage".to_string(),
            cargo_tools::select_package(data.clone()),
        ),
        (
            "cargo-tools.selectBuildTarget".to_string(),
            cargo_tools::select_build_target(data.clone()),
        ),
        (
            "cargo-tools.selectRunTarget".to_string(),
            cargo_tools::select_run_target(data.clone()),
        ),
        (
            "cargo-tools.selectBenchmarkTarget".to_string(),
            cargo_tools::select_benchmark_target(data.clone()),
        ),
        (
            "cargo-tools.selectPlatformTarget".to_string(),
            cargo_tools::select_platform_target(data.clone()),
        ),
        (
            "cargo-tools.installPlatformTarget".to_string(),
            cargo_tools::install_platform_target(data.clone()),
        ),
        (
            "cargo-tools.setRustAnalyzerCheckTargets".to_string(),
            cargo_tools::set_rust_analyzer_check_targets(data.clone()),
        ),
        (
            "cargo-tools.buildDocs".to_string(),
            cargo_tools::build_docs(data.clone()),
        ),
        (
            "cargo-tools.selectFeatures".to_string(),
            cargo_tools::select_features(data.clone()),
        ),
        (
            "cargo-tools.refresh".to_string(),
            cargo_tools::refresh(data.clone()),
        ),
        (
            "cargo-tools.clean".to_string(),
            cargo_tools::clean(data.clone()),
        ),
        (
            "cargo-tools.projectStatus.build".to_string(),
            cargo_tools::project_status::build(data.clone()),
        ),
        (
            "cargo-tools.projectStatus.run".to_string(),
            cargo_tools::project_status::run(data.clone()),
        ),
        (
            "cargo-tools.projectStatus.debug".to_string(),
            cargo_tools::project_status::debug(data.clone()),
        ),
        (
            "cargo-tools.projectStatus.test".to_string(),
            cargo_tools::project_status::test(data.clone()),
        ),
        (
            "cargo-tools.projectStatus.bench".to_string(),
            cargo_tools::project_status::bench(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.selectPackage".to_string(),
            cargo_tools::project_outline::select_package(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.unselectPackage".to_string(),
            cargo_tools::project_outline::unselect_package(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.setBuildTarget".to_string(),
            cargo_tools::project_outline::set_build_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.unsetBuildTarget".to_string(),
            cargo_tools::project_outline::unset_build_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.setRunTarget".to_string(),
            cargo_tools::project_outline::set_run_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.unsetRunTarget".to_string(),
            cargo_tools::project_outline::unset_run_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.setBenchmarkTarget".to_string(),
            cargo_tools::project_outline::set_benchmark_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.unsetBenchmarkTarget".to_string(),
            cargo_tools::project_outline::unset_benchmark_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.buildPackage".to_string(),
            cargo_tools::project_outline::build_package(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.testPackage".to_string(),
            cargo_tools::project_outline::test_package(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.cleanPackage".to_string(),
            cargo_tools::project_outline::clean_package(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.buildWorkspace".to_string(),
            cargo_tools::project_outline::build_workspace(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.testWorkspace".to_string(),
            cargo_tools::project_outline::test_workspace(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.cleanWorkspace".to_string(),
            cargo_tools::project_outline::clean_workspace(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.buildTarget".to_string(),
            cargo_tools::project_outline::build_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.runTarget".to_string(),
            cargo_tools::project_outline::run_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.debugTarget".to_string(),
            cargo_tools::project_outline::debug_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.benchTarget".to_string(),
            cargo_tools::project_outline::bench_target(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.setWorkspaceMemberFilter".to_string(),
            cargo_tools::project_outline::set_workspace_member_filter(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.editWorkspaceMemberFilter".to_string(),
            cargo_tools::project_outline::edit_workspace_member_filter(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.clearWorkspaceMemberFilter".to_string(),
            cargo_tools::project_outline::clear_workspace_member_filter(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.showTargetTypeFilter".to_string(),
            cargo_tools::project_outline::show_target_type_filter(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.clearTargetTypeFilter".to_string(),
            cargo_tools::project_outline::clear_target_type_filter(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.clearAllFilters".to_string(),
            cargo_tools::project_outline::clear_all_filters(data.clone()),
        ),
        (
            "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping".to_string(),
            cargo_tools::project_outline::toggle_workspace_member_grouping(data.clone()),
        ),
    ])
}

fn cargo_make_command_map(tx: Sender<CargoMakeMsg>, _data: cargo_make::CommandData) -> CommandMap {
    HashMap::from([
        (
            "cargo-tools.makefile.runTask".to_string(),
            cargo_tools::makefile::run_task(tx.clone()),
        ),
        (
            "cargo-tools.makefile.selectAndRunTask".to_string(),
            cargo_tools::makefile::select_and_run_task(tx.clone()),
        ),
        (
            "cargo-tools.makefile.setTaskFilter".to_string(),
            cargo_tools::makefile::set_task_filter(tx.clone()),
        ),
        (
            "cargo-tools.makefile.editTaskFilter".to_string(),
            cargo_tools::makefile::edit_task_filter(tx.clone()),
        ),
        (
            "cargo-tools.makefile.clearTaskFilter".to_string(),
            cargo_tools::makefile::clear_task_filter(tx.clone()),
        ),
        (
            "cargo-tools.makefile.showCategoryFilter".to_string(),
            cargo_tools::makefile::show_category_filter(tx.clone()),
        ),
        (
            "cargo-tools.makefile.clearCategoryFilter".to_string(),
            cargo_tools::makefile::clear_category_filter(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.add".to_string(),
            cargo_tools::pinned_makefile_tasks::add(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.remove".to_string(),
            cargo_tools::pinned_makefile_tasks::remove(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute".to_string(),
            cargo_tools::pinned_makefile_tasks::execute(tx.clone()),
        ),
        (
            "cargo-tools.makefile.pinTask".to_string(),
            cargo_tools::makefile::pin_task(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute1".to_string(),
            cargo_tools::pinned_makefile_tasks::execute1(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute2".to_string(),
            cargo_tools::pinned_makefile_tasks::execute2(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute3".to_string(),
            cargo_tools::pinned_makefile_tasks::execute3(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute4".to_string(),
            cargo_tools::pinned_makefile_tasks::execute4(tx.clone()),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute5".to_string(),
            cargo_tools::pinned_makefile_tasks::execute5(tx.clone()),
        ),
    ])
}

#[derive(Debug)]
pub struct SelectInput<T: ToQuickPickItem + Debug> {
    pub options: Vec<T>,
    pub current: Vec<T>,
}
#[cfg(test)]
pub mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::contributes::data::all_commands;

    #[wasm_bindgen_test]
    fn all_commands_are_registered() {
        let (cargo_make_tx, _rx) = async_broadcast::broadcast(10);
        let closures = {
            let mut cmds = cargo_command_map(todo!());
            cmds.extend(cargo_make_command_map(cargo_make_tx, todo!()));
            cmds
        };
        let commands = all_commands();

        for cmd in commands {
            assert!(
                closures.contains_key(&cmd.command),
                "Command '{}' from all_commands() was not registered in COMMAND_CLOSURES",
                cmd.command
            );
        }
    }
}
