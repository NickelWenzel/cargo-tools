#![cfg(not(target_arch = "wasm32"))]

//! Native integration test for headless applications.

use futures::SinkExt;
use iced::{stream, Subscription, Task};
use iced_headless::{application, event_loop::Exit, tokio_context::TokioContext};

#[derive(Debug, Clone)]
struct Message;

struct SimpleProgram {
    tx: Option<tokio::sync::mpsc::Sender<Message>>,
}

impl SimpleProgram {
    fn new(tx: tokio::sync::mpsc::Sender<Message>) -> (Self, Task<Message>) {
        (Self { tx: Some(tx) }, Task::done(Message))
    }
}

impl SimpleProgram {
    fn update(&mut self, msg: Message) -> Task<Message> {
        if let Some(tx) = self.tx.take() {
            return Task::future(async move { tx.send(msg).await }).map(|_| Message);
        };

        Task::done(Message)
    }

    fn exit(&self) -> Subscription<Exit> {
        if self.tx.is_some() {
            return Subscription::none();
        }
        Subscription::run(|| {
            stream::channel(1, |mut tx| async move {
                let _ = tx.send(Exit).await;
            })
        })
    }
}

#[tokio::test]
async fn native_simple_completes() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    application(SimpleProgram::update)
        .executor::<TokioContext>()
        .exit_on(SimpleProgram::exit)
        .run_with(|| SimpleProgram::new(tx))
        .await
        .unwrap();

    assert!((rx.recv().await).is_some());
}
