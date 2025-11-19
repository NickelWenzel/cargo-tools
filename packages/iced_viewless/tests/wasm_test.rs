//! WASM integration test for viewless applications.

#![cfg(target_arch = "wasm32")]

use iced_futures::Subscription;
use iced_viewless::{application, ViewlessProgram};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Clone)]
enum Message {
    Done,
}

#[derive(Default)]
struct WasmProgram;

#[derive(Debug, Default, Clone)]
struct State {
    completed: bool,
}

impl ViewlessProgram for WasmProgram {
    type State = State;
    type Message = Message;
    type Executor = iced_futures::backend::wasm::wasm_bindgen::Executor;

    fn update(&self, state: &mut Self::State, _message: Self::Message) {
        state.completed = true;
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        if state.completed {
            Subscription::none()
        } else {
            Subscription::run_with_id("once", futures::stream::iter(vec![Message::Done]))
        }
    }
}

#[wasm_bindgen_test]
async fn wasm_simple_completes() {
    use iced_futures::Executor;

    let executor = iced_futures::backend::wasm::wasm_bindgen::Executor::new()
        .expect("Failed to create WASM executor");

    application(WasmProgram::default())
        .run_with(|| State { completed: false }, executor)
        .await
        .expect("WASM program should complete successfully");
}
