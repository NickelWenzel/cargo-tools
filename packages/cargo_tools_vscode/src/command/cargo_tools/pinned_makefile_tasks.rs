use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::app::CargoMakeMsg;

pub fn add(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn remove(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn execute(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn execute1(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn execute2(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn execute3(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn execute4(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn execute5(_tx: Sender<CargoMakeMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
