use futures::channel::mpsc::Receiver;
use wasm_async_trait::wasm_async_trait;

use crate::app::state::{State, StateUpdate};

#[derive(Debug, Clone)]
pub struct Settings;

#[derive(Debug, Clone)]
pub struct SettingsUpdate;

#[wasm_async_trait]
pub trait Context: 'static {
    async fn update_state_context(ctx: String);
    async fn update_state(update: StateUpdate);
    fn state_receiver() -> Receiver<State>;

    async fn update_settings_context(ctx: String);
    async fn update_settings(update: SettingsUpdate);
    fn settings_receiver() -> Receiver<Settings>;
}
