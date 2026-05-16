use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    commands::cargo_make::*,
    extension::{CommandBinding, CommandMap, register_tasks},
    vs_code_api::{log_error, log_info, try_get_task_label},
};

#[derive(Debug, Clone)]
pub enum Command {
    RunTask(String),
    SelectAndRunTask,
    SelectTaskFilter,
    EditTaskFilter(String),
    SelectCategoryFilter,
    EditCategoryFilter(Vec<String>),
    ClearAllFilters,
    PinTask(String),
}

impl Command {
    pub const fn all() -> [(&'static str, CargoMakeCmdFn); NUMBER_CMDS] {
        [
            (CARGO_TOOLS_MAKEFILE_RUNTASK, |arg| {
                try_get_task_label(arg).map(Self::RunTask)
            }),
            (CARGO_TOOLS_MAKEFILE_SELECTANDRUNTASK, |_| {
                Some(Self::SelectAndRunTask)
            }),
            (CARGO_TOOLS_MAKEFILE_SELECTTASKFILTER, |_| {
                Some(Self::SelectTaskFilter)
            }),
            (CARGO_TOOLS_MAKEFILE_SELECTCATEGORYFILTER, |_| {
                Some(Self::SelectCategoryFilter)
            }),
            (CARGO_TOOLS_MAKEFILE_CLEARALLFILTERS, |_| {
                Some(Self::ClearAllFilters)
            }),
            (CARGO_TOOLS_MAKEFILE_PINTASK, |arg| {
                try_get_task_label(arg).map(Self::PinTask)
            }),
        ]
    }
}

pub type CargoMakeCmdFn = fn(Array) -> Option<Command>;

pub fn register_cargo_make_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, CommandBinding);

fn create_vs_code_command(
    tx: Sender<Command>,
    key: &'static str,
    cargo_cmd_fn: CargoMakeCmdFn,
) -> CmdKeyValuePair {
    let cmd = Closure::new(move |args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            let Some(cmd) = cargo_cmd_fn(args) else {
                log_error("Failed to extract cargo make command");
                return;
            };
            log_info(&format!("Sending VS Code cargo make command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log_error(&format!("Failed to queue msg: {}", e));
            }
        });
    });

    (key, cmd)
}

pub fn task_map(tx: Sender<Command>) -> CommandMap {
    HashMap::from(
        Command::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    )
}
