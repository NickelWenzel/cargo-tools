use std::collections::HashMap;

use cargo_tools::cargo_make::tasks::MakefileTask;
use futures::{SinkExt, channel::mpsc::Sender};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    extension::{TaskMap, VsCodeTask, cargo_make::ui::CargoMakeNodeHandler, register_tasks},
    vs_code_api::{log, try_get_handler},
};

pub mod process;

#[derive(Debug, Clone)]
pub enum Command {
    RunTask(MakefileTask),
    SelectAndRunTask,
    SelectTaskFilter,
    EditTaskFilter(String),
    SelectCategoryFilter,
    EditCategoryFilter(Vec<String>),
    ClearAllFilters,
    PinTask(MakefileTask),
    Pinned(Pinned),
}

impl Command {
    pub const fn all() -> [(&'static str, CargoMakeCmdFn); 14] {
        [
            ("cargo-tools.makefile.runTask", |arg| {
                try_task_from_node(arg, Self::RunTask)
            }),
            ("cargo-tools.makefile.selectAndRunTask", |_| {
                Some(Self::SelectAndRunTask)
            }),
            ("cargo-tools.makefile.selectTaskFilter", |_| {
                Some(Self::SelectTaskFilter)
            }),
            ("cargo-tools.makefile.selectCategoryFilter", |_| {
                Some(Self::SelectCategoryFilter)
            }),
            ("cargo-tools.makefile.clearAllFilters", |_| {
                Some(Self::ClearAllFilters)
            }),
            ("cargo-tools.makefile.pinTask", |arg| {
                try_task_from_node(arg, Self::PinTask)
            }),
            ("cargo-tools.pinnedMakefileTasks.add", |_| {
                Some(Self::Pinned(Pinned::Add))
            }),
            ("cargo-tools.pinnedMakefileTasks.remove", |arg| {
                try_task_from_node(arg, Pinned::Remove).map(Self::Pinned)
            }),
            ("cargo-tools.pinnedMakefileTasks.execute", |arg| {
                try_task_from_node(arg, Pinned::Execute).map(Self::Pinned)
            }),
            ("cargo-tools.pinnedMakefileTasks.execute1", |_| {
                Some(Self::Pinned(Pinned::Execute1))
            }),
            ("cargo-tools.pinnedMakefileTasks.execute2", |_| {
                Some(Self::Pinned(Pinned::Execute2))
            }),
            ("cargo-tools.pinnedMakefileTasks.execute3", |_| {
                Some(Self::Pinned(Pinned::Execute3))
            }),
            ("cargo-tools.pinnedMakefileTasks.execute4", |_| {
                Some(Self::Pinned(Pinned::Execute4))
            }),
            ("cargo-tools.pinnedMakefileTasks.execute5", |_| {
                Some(Self::Pinned(Pinned::Execute5))
            }),
        ]
    }
}

pub type CargoMakeCmdFn = fn(Array) -> Option<Command>;

#[derive(Debug, Clone)]
pub enum Pinned {
    Add,
    Remove(MakefileTask),
    Execute(MakefileTask),
    Execute1,
    Execute2,
    Execute3,
    Execute4,
    Execute5,
}

fn try_task_from_node<To>(arg: Array, cmd: fn(MakefileTask) -> To) -> Option<To> {
    try_get_handler(arg)
        .and_then(CargoMakeNodeHandler::try_into_task)
        .map(cmd)
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
