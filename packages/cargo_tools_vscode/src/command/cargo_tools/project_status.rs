use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::command::CargoCmdData;

pub fn build(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn run(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn debug(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn bench(_data: CargoCmdData) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
