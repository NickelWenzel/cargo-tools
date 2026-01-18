use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::command::CargoCmdData;

pub fn select_package(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unselect_package(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_build_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unset_build_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_run_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unset_run_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_benchmark_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unset_benchmark_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_package(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test_package(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clean_package(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_workspace(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test_workspace(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clean_workspace(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn run_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn debug_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn bench_target(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_workspace_member_filter(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn edit_workspace_member_filter(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_workspace_member_filter(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn show_target_type_filter(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_target_type_filter(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_all_filters(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn toggle_workspace_member_grouping(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
