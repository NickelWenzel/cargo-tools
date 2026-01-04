use std::collections::HashMap;

use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    app::{Message, cargo, cargo_make},
    vs_code_api::{log, register_command},
};

mod cargo_tools;

pub fn register_commands(tx: Sender<Message>, cargo_ui: cargo::Ui, cargo_make_ui: cargo_make::Ui) {
    for (command_id, closure) in command_map(tx, cargo_ui, cargo_make_ui) {
        if let Err(e) = register_command(&command_id, &closure) {
            log(&format!(
                "Failed to register command '{}': {:?}",
                command_id, e
            ));
        }
    }
}

type CommandMap = HashMap<String, Closure<dyn FnMut(Array)>>;

fn command_map(
    tx: Sender<Message>,
    cargo_ui: cargo::Ui,
    _cargo_make_ui: cargo_make::Ui,
) -> CommandMap {
    HashMap::from([
        (
            "cargo-tools.selectProfile".to_string(),
            cargo_tools::select_profile(tx.clone(), cargo_ui.data.clone()),
        ),
        (
            "cargo-tools.selectPackage".to_string(),
            cargo_tools::select_package(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.selectBuildTarget".to_string(),
            cargo_tools::select_build_target(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.selectRunTarget".to_string(),
            cargo_tools::select_run_target(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.selectBenchmarkTarget".to_string(),
            cargo_tools::select_benchmark_target(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.selectPlatformTarget".to_string(),
            cargo_tools::select_platform_target(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.installPlatformTarget".to_string(),
            cargo_tools::install_platform_target(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.setRustAnalyzerCheckTargets".to_string(),
            cargo_tools::set_rust_analyzer_check_targets(tx.clone()),
        ),
        (
            "cargo-tools.buildDocs".to_string(),
            cargo_tools::build_docs(tx.clone()),
        ),
        (
            "cargo-tools.selectFeatures".to_string(),
            cargo_tools::select_features(tx.clone(), cargo_ui.clone()),
        ),
        (
            "cargo-tools.refresh".to_string(),
            cargo_tools::refresh(tx.clone()),
        ),
        (
            "cargo-tools.clean".to_string(),
            cargo_tools::clean(tx.clone(), cargo_ui.clone()),
        ),
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
        (
            "cargo-tools.projectStatus.build".to_string(),
            cargo_tools::project_status::build(tx.clone()),
        ),
        (
            "cargo-tools.projectStatus.run".to_string(),
            cargo_tools::project_status::run(tx.clone()),
        ),
        (
            "cargo-tools.projectStatus.debug".to_string(),
            cargo_tools::project_status::debug(tx.clone()),
        ),
        (
            "cargo-tools.projectStatus.test".to_string(),
            cargo_tools::project_status::test(tx.clone()),
        ),
        (
            "cargo-tools.projectStatus.bench".to_string(),
            cargo_tools::project_status::bench(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.selectPackage".to_string(),
            cargo_tools::project_outline::select_package(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.unselectPackage".to_string(),
            cargo_tools::project_outline::unselect_package(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.setBuildTarget".to_string(),
            cargo_tools::project_outline::set_build_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.unsetBuildTarget".to_string(),
            cargo_tools::project_outline::unset_build_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.setRunTarget".to_string(),
            cargo_tools::project_outline::set_run_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.unsetRunTarget".to_string(),
            cargo_tools::project_outline::unset_run_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.setBenchmarkTarget".to_string(),
            cargo_tools::project_outline::set_benchmark_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.unsetBenchmarkTarget".to_string(),
            cargo_tools::project_outline::unset_benchmark_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.buildPackage".to_string(),
            cargo_tools::project_outline::build_package(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.testPackage".to_string(),
            cargo_tools::project_outline::test_package(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.cleanPackage".to_string(),
            cargo_tools::project_outline::clean_package(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.buildWorkspace".to_string(),
            cargo_tools::project_outline::build_workspace(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.testWorkspace".to_string(),
            cargo_tools::project_outline::test_workspace(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.cleanWorkspace".to_string(),
            cargo_tools::project_outline::clean_workspace(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.buildTarget".to_string(),
            cargo_tools::project_outline::build_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.runTarget".to_string(),
            cargo_tools::project_outline::run_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.debugTarget".to_string(),
            cargo_tools::project_outline::debug_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.benchTarget".to_string(),
            cargo_tools::project_outline::bench_target(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.setWorkspaceMemberFilter".to_string(),
            cargo_tools::project_outline::set_workspace_member_filter(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.editWorkspaceMemberFilter".to_string(),
            cargo_tools::project_outline::edit_workspace_member_filter(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.clearWorkspaceMemberFilter".to_string(),
            cargo_tools::project_outline::clear_workspace_member_filter(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.showTargetTypeFilter".to_string(),
            cargo_tools::project_outline::show_target_type_filter(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.clearTargetTypeFilter".to_string(),
            cargo_tools::project_outline::clear_target_type_filter(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.clearAllFilters".to_string(),
            cargo_tools::project_outline::clear_all_filters(tx.clone()),
        ),
        (
            "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping".to_string(),
            cargo_tools::project_outline::toggle_workspace_member_grouping(tx.clone()),
        ),
    ])
}

#[cfg(test)]
pub mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::contributes::data::all_commands;

    #[wasm_bindgen_test]
    fn all_commands_are_registered() {
        let (tx, _rx) = async_broadcast::broadcast(10);
        let closures = command_map(tx, cargo::Ui::new(), cargo_make::Ui::new());
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
