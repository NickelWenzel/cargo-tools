use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::app::CargoMsg;

pub fn build(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn run(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn debug(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn bench(_tx: Sender<CargoMsg>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
