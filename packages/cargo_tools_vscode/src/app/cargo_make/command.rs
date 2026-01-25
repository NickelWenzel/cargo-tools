use wasm_bindgen_futures::js_sys::Array;

pub mod register;

#[derive(Debug, Clone)]
pub enum Command {}

pub type CargoCmdFn = fn(Array) -> Option<Command>;
