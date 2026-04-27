use std::collections::HashMap;

use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    commands::cargo_make::*,
    extension::{TaskMap, VsCodeTask, register_tasks},
    vs_code_api::{log, try_get_task_label},
};

pub mod process;

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
    Pinned(Pinned),
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
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_ADD, |_| {
                Some(Self::Pinned(Pinned::Add))
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_REMOVE, |arg| {
                try_get_task_label(arg)
                    .map(Pinned::Remove)
                    .map(Self::Pinned)
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_EXECUTE, |arg| {
                try_get_task_label(arg)
                    .map(Pinned::Execute)
                    .map(Self::Pinned)
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_EXECUTE1, |_| {
                Some(Self::Pinned(Pinned::Execute1))
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_EXECUTE2, |_| {
                Some(Self::Pinned(Pinned::Execute2))
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_EXECUTE3, |_| {
                Some(Self::Pinned(Pinned::Execute3))
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_EXECUTE4, |_| {
                Some(Self::Pinned(Pinned::Execute4))
            }),
            (CARGO_TOOLS_PINNEDMAKEFILETASKS_EXECUTE5, |_| {
                Some(Self::Pinned(Pinned::Execute5))
            }),
        ]
    }
}

pub type CargoMakeCmdFn = fn(Array) -> Option<Command>;

#[derive(Debug, Clone)]
pub enum Pinned {
    Add,
    Remove(String),
    Execute(String),
    Execute1,
    Execute2,
    Execute3,
    Execute4,
    Execute5,
}

pub fn register_cargo_make_commands(tx: Sender<Command>) -> Vec<VsCodeTask> {
    register_tasks(task_map(tx))
}

type CmdKeyValuePair = (&'static str, VsCodeTask);

fn create_vs_code_command(
    tx: Sender<Command>,
    key: &'static str,
    cargo_cmd_fn: CargoMakeCmdFn,
) -> CmdKeyValuePair {
    let cmd = Closure::new(move |args: Array| {
        let tx = tx.clone();
        spawn_local(async move {
            let Some(cmd) = cargo_cmd_fn(args) else {
                log("Failed to extract cargo make command");
                return;
            };
            log(&format!("Sending VS Code cargo make command '{cmd:?}'"));
            if let Err(e) = tx.clone().send(cmd).await {
                log(&format!("Failed to queue msg: {}", e));
            }
        });
    });

    (key, cmd)
}

pub fn task_map(tx: Sender<Command>) -> TaskMap {
    HashMap::from(
        Command::all().map(|(key, cmd_fn)| create_vs_code_command(tx.clone(), key, cmd_fn)),
    )
}
