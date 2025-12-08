use std::fmt::Debug;

use futures::{
    channel::mpsc::{self, UnboundedSender},
    StreamExt,
};
use iced_runtime::Action;
use log::info;

pub struct Exit;

pub struct EventLoop<T> {
    rx: mpsc::UnboundedReceiver<Action<T>>,
}

impl<T> EventLoop<T> {
    pub fn new() -> (UnboundedSender<Action<T>>, Self) {
        let (tx, rx) = mpsc::unbounded();
        (tx, Self { rx })
    }

    pub async fn run<State>(mut self, mut state: State, mut f: impl FnMut(&mut State, T))
    where
        T: Debug,
    {
        while let Some(action) = self.rx.next().await {
            info!("Received action {action:?}");
            match action {
                Action::Output(message) => f(&mut state, message),
                Action::Exit => break,
                _ => continue,
            }
        }
    }
}
