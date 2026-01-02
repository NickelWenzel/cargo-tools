use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::vs_code_api::{log, register_command};

pub fn register_commands() {
    for (command_id, closure) in commmand_map() {
        if let Err(e) = register_command(&command_id, &closure) {
            log(&format!(
                "Failed to register command '{}': {:?}",
                command_id, e
            ));
        }
    }
}

type CommandMap = HashMap<String, Closure<dyn FnMut(Array)>>;

fn commmand_map() -> CommandMap {
    HashMap::from([
        (
            "cargo-tools.selectProfile".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.selectPackage".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.selectBuildTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.selectRunTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.selectBenchmarkTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.selectPlatformTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.installPlatformTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.setRustAnalyzerCheckTargets".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.buildDocs".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.selectFeatures".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.refresh".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.clean".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.runTask".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.selectAndRunTask".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.setTaskFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.editTaskFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.clearTaskFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.showCategoryFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.clearCategoryFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.add".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.remove".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.makefile.pinTask".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute1".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute2".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute3".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute4".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.pinnedMakefileTasks.execute5".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectStatus.build".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectStatus.run".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectStatus.debug".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectStatus.test".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectStatus.bench".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.selectPackage".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.unselectPackage".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.setBuildTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.unsetBuildTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.setRunTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.unsetRunTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.setBenchmarkTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.unsetBenchmarkTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.buildPackage".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.testPackage".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.cleanPackage".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.buildWorkspace".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.testWorkspace".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.cleanWorkspace".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.buildTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.runTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.debugTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.benchTarget".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.setWorkspaceMemberFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.editWorkspaceMemberFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.clearWorkspaceMemberFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.showTargetTypeFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.clearTargetTypeFilter".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.clearAllFilters".to_string(),
            Closure::new(|_args: Array| {}),
        ),
        (
            "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping".to_string(),
            Closure::new(|_args: Array| {}),
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
        let closures = commmand_map();
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
