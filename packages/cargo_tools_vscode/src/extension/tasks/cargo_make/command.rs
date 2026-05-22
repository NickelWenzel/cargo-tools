use futures::channel::mpsc::Sender;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::cargo_make::*,
    extension::vscode_task_utils::{CommandBinding, register_commands},
    vs_code_api::try_get_task_label,
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

type CmdFn = fn(Array) -> Option<Command>;

impl Command {
    const fn all() -> [(&'static str, CmdFn); NUMBER_CMDS] {
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

pub fn register_cargo_make_commands(tx: Sender<Command>) -> Vec<CommandBinding> {
    register_commands(tx, Command::all())
}
