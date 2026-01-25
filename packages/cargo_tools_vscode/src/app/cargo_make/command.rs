use cargo_tools::app::cargo_make::tasks::MakefileTask;
use serde::de::DeserializeOwned;
use wasm_bindgen_futures::js_sys::Array;

use crate::vs_code_api::log;

pub mod register;

#[derive(Debug, Clone)]
pub enum Command {
    RunTask(String),
    SelectAndRunTask,
    SelectTaskFilter,
    EditTaskFilter(String),
    SelectCategoryFilter,
    EditCategoryFilter(String),
    ClearAllFilters,
    PinTask(MakefileTask),
    Pinned(Pinned),
}

impl Command {
    pub const fn all() -> [(&'static str, CargoMakeCmdFn); 16] {
        [
            ("cargo-tools.makefile.runTask", |arg| {
                try_first_into(arg, Self::RunTask)
            }),
            ("cargo-tools.makefile.selectAndRunTask", |_| {
                Some(Self::SelectAndRunTask)
            }),
            ("cargo-tools.makefile.selectTaskFilter", |_| {
                Some(Self::SelectTaskFilter)
            }),
            ("cargo-tools.makefile.editTaskFilter", |arg| {
                try_first_into(arg, Self::EditTaskFilter)
            }),
            ("cargo-tools.makefile.selectCategoryFilter", |_| {
                Some(Self::SelectCategoryFilter)
            }),
            ("cargo-tools.makefile.editCategoryFilter", |arg| {
                try_first_into(arg, Self::EditCategoryFilter)
            }),
            ("cargo-tools.makefile.clearAllFilters", |_| {
                Some(Self::ClearAllFilters)
            }),
            ("cargo-tools.makefile.pinTask", |arg| {
                try_first_into(arg, Self::PinTask)
            }),
            ("cargo-tools.pinnedMakefileTasks.add", |_| {
                Some(Self::Pinned(Pinned::Add))
            }),
            ("cargo-tools.pinnedMakefileTasks.remove", |arg| {
                try_first_into(arg, Pinned::Remove).map(Self::Pinned)
            }),
            ("cargo-tools.pinnedMakefileTasks.execute", |arg| {
                try_first_into(arg, Pinned::Execute).map(Self::Pinned)
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
    Remove(usize),
    Execute(String),
    Execute1,
    Execute2,
    Execute3,
    Execute4,
    Execute5,
}

fn try_first_into<T: DeserializeOwned, To>(arg: Array, cmd: fn(T) -> To) -> Option<To> {
    take_first(arg).map(cmd)
}

fn take_first<T: DeserializeOwned>(array: Array) -> Option<T> {
    match serde_wasm_bindgen::from_value(array.get(0)) {
        Ok(v) => Some(v),
        Err(e) => {
            log(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}
