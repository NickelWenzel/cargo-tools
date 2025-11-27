//! WASM integration test for headless applications.

#![cfg(target_arch = "wasm32")]

use std::dbg;

use iced::{Subscription, Task};
use iced_headless::{application, event_loop::Exit};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

#[derive(Debug, Default, Clone)]
struct State {
    count: u32,
}

struct WasmProgram;

impl WasmProgram {
    fn update(state: &mut State, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                state.count += 1;
                dbg!("Incrementing");
                Task::none()
            }
        }
    }

    fn subscription(state: &State) -> Subscription<Message> {
        // Immediately provide a message to increment
        if state.count == 0 {
            dbg!("Dispatch increment");
            Subscription::run_with_id("once", futures::stream::iter(vec![Message::Increment]))
        } else {
            Subscription::none()
        }
    }

    fn exit(state: &State) -> Subscription<Exit> {
        // Exit after we've incremented once
        if state.count > 0 {
            dbg!("Dispatch exit");
            Subscription::run(|| futures::stream::once(async { Exit }))
        } else {
            Subscription::none()
        }
    }
}

#[wasm_bindgen_test]
// #[ignore = "WASM browser test requires manual verification - times out in automated testing"]
async fn wasm_simple_increment() {
    let result = application(WasmProgram::update)
        .subscription(|state| WasmProgram::subscription(state))
        .exit_on(|state| WasmProgram::exit(state))
        .run_with(|| (State::default(), Task::none()))
        .await;

    assert!(result.is_ok(), "WASM program should complete successfully");
}
