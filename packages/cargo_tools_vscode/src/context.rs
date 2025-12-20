use cargo_tools::app::state::{State, StateUpdate};
use cargo_tools::context::{Configuration, ConfigurationUpdate, Context};
use futures::channel::mpsc::Receiver;
use wasm_async_trait::wasm_async_trait;

pub struct VsCodeContext;

#[wasm_async_trait]
impl Context for VsCodeContext {
    async fn update_state_context(_ctx: String) {
        todo!()
    }

    async fn update_state(_update: StateUpdate) {
        todo!()
    }

    fn state_receiver() -> Receiver<State> {
        todo!()
    }

    async fn update_configuration_context(_ctx: String) {
        todo!()
    }

    async fn update_configuration(_update: ConfigurationUpdate) {
        todo!()
    }

    fn configuration_receiver() -> Receiver<Configuration> {
        todo!()
    }
}
