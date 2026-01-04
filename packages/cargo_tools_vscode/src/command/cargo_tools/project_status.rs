use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::app::Message;

pub fn build(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn run(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn debug(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn test(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn bench(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
