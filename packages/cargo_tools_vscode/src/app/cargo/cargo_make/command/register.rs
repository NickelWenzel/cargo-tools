use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    app::{
        CargoMakeMsg, Command, CommandMap,
        cargo::VsCodeCargoCmd::{self, *},
    },
    vs_code_api::{log, register_command},
};

pub fn register_cargo_commands(tx: Sender<VsCodeCargoCmd>) -> Vec<Command> {
    register_commands(cargo_command_map(tx))
}

pub fn register_cargo_make_commands(tx: Sender<CargoMakeMsg>) -> Vec<Command> {
    register_commands(cargo_make_command_map(tx))
}

fn register_commands(cmds: CommandMap) -> Vec<Command> {
    cmds.into_iter()
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

type CmdKeyValuePair = (&'static str, Command);

fn create_command(tx: Sender<VsCodeCargoCmd>, cmd: VsCodeCargoCmd) -> CmdKeyValuePair {
    let key = cmd.cmd_key();
    let cmd = Closure::new(move |_args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            log(&format!("Sending VS Code cargo command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    });

    (key, cmd)
}

fn cargo_command_map(tx: Sender<VsCodeCargoCmd>) -> CommandMap {
    HashMap::from(VsCodeCargoCmd::all().map(|cmd| create_command(tx.clone(), cmd)))
}

fn cargo_make_command_map(tx: Sender<CargoMakeMsg>) -> CommandMap {
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

#[cfg(test)]
pub mod tests {
    use futures::channel::mpsc::channel;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::contributes::data::all_commands;

    #[wasm_bindgen_test]
    fn all_commands_are_registered() {
        let (cargo_tx, _rx) = channel(10);
        let (cargo_make_tx, _rx) = channel(10);
        let closures = {
            let mut cmds = cargo_command_map(cargo_tx);
            cmds.extend(cargo_make_command_map(cargo_make_tx));
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
