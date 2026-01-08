use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::app::CargoMsg;

pub fn select_package(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unselect_package(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_build_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unset_build_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_run_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unset_run_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_benchmark_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn unset_benchmark_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_package(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test_package(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clean_package(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_workspace(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test_workspace(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clean_workspace(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn build_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn run_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn debug_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn bench_target(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_workspace_member_filter(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn edit_workspace_member_filter(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_workspace_member_filter(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn show_target_type_filter(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_target_type_filter(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_all_filters(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn toggle_workspace_member_grouping(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
