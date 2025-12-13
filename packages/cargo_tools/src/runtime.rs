use std::future::Future;

use async_broadcast::Receiver;
use wasm_async_trait::wasm_async_trait;

use crate::{
    cargo_tools::{Settings, SettingsUpdate},
    state::{State, StateUpdate},
};

#[wasm_async_trait]
pub trait Runtime {
    type ThreadHandle: Future;

    fn spawn<Result, F>(f: F) -> Self::ThreadHandle
    where
        Self::ThreadHandle: Future<Output = Result>;

    async fn exec(command: String) -> Result<String, String>;
    async fn log(msg: String);

    async fn current_dir_notitifier() -> Receiver<String>;

    async fn update_state_context(ctx: String) -> State;
    async fn update_state(update: StateUpdate) -> State;

    async fn update_settings_context(ctx: String) -> Settings;
    async fn update_settings(update: SettingsUpdate) -> Settings;
}
