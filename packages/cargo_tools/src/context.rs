use async_broadcast::Receiver;
use wasm_async_trait::wasm_async_trait;

use crate::app::state::{State, StateUpdate};

pub use crate::contributes::Configuration;

#[derive(Debug, Clone)]
pub struct ConfigurationUpdate {
    pub key: String,
    pub property: crate::contributes::ConfigurationProperty,
}

#[wasm_async_trait]
pub trait Context: 'static {
    fn update_prefix(prefix: String);

    async fn update_state(update: StateUpdate);
    fn state_receiver() -> Receiver<State>;
}
