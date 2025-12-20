use futures::channel::mpsc::Receiver;
use wasm_async_trait::wasm_async_trait;

use crate::app::state::{State, StateUpdate};

#[derive(Debug, Clone)]
pub struct Configuration;

#[derive(Debug, Clone)]
pub struct ConfigurationUpdate;

#[wasm_async_trait]
pub trait Context: 'static {
    async fn update_state_context(ctx: String);
    async fn update_state(update: StateUpdate);
    fn state_receiver() -> Receiver<State>;

    async fn update_configuration_context(ctx: String);
    async fn update_configuration(update: ConfigurationUpdate);
    fn configuration_receiver() -> Receiver<Configuration>;
}
