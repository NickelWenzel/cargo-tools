use iced::{stream, Subscription, Task};
use iced_headless::{application, event_loop::Exit};
use log::info;
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
        Task::done(Message)
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
fn wasm_simple_completes() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).unwrap();

    info!("InTest");
    application(SimpleProgram::update)
        .exit_on(SimpleProgram::exit)
        .run_with(SimpleProgram::new)
        .unwrap();
    info!("EndTest");
}
