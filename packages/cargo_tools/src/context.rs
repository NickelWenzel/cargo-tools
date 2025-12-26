use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use wasm_async_trait::wasm_async_trait;

use crate::contributes::Configuration;

#[wasm_async_trait]
pub trait Context: 'static {
    async fn persist_state(prefix: String, state: impl Serialize);
    fn get_state<T: DeserializeOwned + Debug>(prefix: String) -> Option<T>;

    fn get_configuration() -> Option<Configuration>;
}
