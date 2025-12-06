use iced::{stream, Subscription, Task};
use iced_headless::{application, event_loop::Exit};
use log::info;
use wasm_bindgen_test::*;

use futures::{SinkExt, StreamExt};

#[derive(Debug, Clone)]
struct Message;

struct SimpleProgram {
    tx: Option<futures::channel::mpsc::Sender<Message>>,
}

impl SimpleProgram {
    fn new(tx: futures::channel::mpsc::Sender<Message>) -> (Self, Task<Message>) {
        info!("In new");
        (Self { tx: Some(tx) }, Task::done(Message))
    }
}

impl SimpleProgram {
    fn update(&mut self, msg: Message) -> Task<Message> {
        info!("In update");
        if let Some(mut tx) = self.tx.take() {
            info!("In update: send message outside");
            return Task::future(async move { tx.send(msg).await }).map(|_| Message);
        };

        info!("In update: send new message");
        Task::none()
    }

    fn exit(&self) -> Subscription<Exit> {
        info!("In exit");
        if self.tx.is_some() {
            info!("In exit: do nothing");
            return Subscription::none();
        }

        info!("In exit: send exit signal");
        Subscription::run(|| {
            stream::channel(1, |mut tx| async move {
                let _ = tx.send(Exit).await;
            })
        })
    }
}

#[wasm_bindgen_test(unsupported = tokio::test)]
async fn wasm_simple_completes() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Debug).unwrap();

    info!("InTest");
    let (tx, mut rx) = futures::channel::mpsc::channel(1);

    application(SimpleProgram::update)
        .exit_on(SimpleProgram::exit)
        .run_with(|| SimpleProgram::new(tx))
        .await
        .unwrap();
    info!("EndTest");

    assert!((rx.next().await).is_some());
}
