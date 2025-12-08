use iced::{stream, Subscription, Task};
use iced_headless::{application, event_loop::Exit, headless::async_application};
use log::info;
use tracing_test::traced_test;
use wasm_bindgen_test::*;

use futures::SinkExt;

#[derive(Debug, Clone)]
struct Message;

struct SimpleProgram {
    already_updated: bool,
}

impl SimpleProgram {
    fn new() -> (Self, Task<Message>) {
        info!("In new");
        (
            Self {
                already_updated: false,
            },
            Task::done(Message),
        )
    }
}

impl SimpleProgram {
    fn update(&mut self, _: Message) -> Task<Message> {
        info!("In update");
        self.already_updated = true;
        Task::none()
    }

    fn exit(&self) -> Subscription<Exit> {
        info!("In exit");
        if self.already_updated {
            info!("In exit: send exit signal");
            Subscription::run(|| {
                stream::channel(1, |mut tx| async move {
                    let _ = tx.send(Exit).await;
                })
            })
        } else {
            info!("In exit: do nothing");
            Subscription::none()
        }
    }
}

#[wasm_bindgen_test(unsupported = test)]
#[traced_test]
fn sync_app_completes() {
    info!("InTest");
    let result = application(SimpleProgram::update)
        .exit_on(SimpleProgram::exit)
        .run_with(SimpleProgram::new);

    assert!(result.is_ok());
    info!("EndTest");
}

#[wasm_bindgen_test(unsupported = tokio::test)]
#[traced_test]
async fn async_app_completes() {
    info!("InTest");
    let result = async_application(SimpleProgram::update)
        .exit_on(SimpleProgram::exit)
        .run_with(SimpleProgram::new)
        .await;

    assert!(result.is_ok());
    info!("EndTest");
}
