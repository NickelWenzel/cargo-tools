use async_broadcast::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Array;

use crate::app::Message;

pub fn run_task(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn select_and_run_task(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn set_task_filter(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn edit_task_filter(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_task_filter(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn show_category_filter(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn clear_category_filter(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}

pub fn pin_task(_tx: Sender<Message>) -> Closure<dyn FnMut(Array)> {
    Closure::new(move |_args: Array| {})
}
