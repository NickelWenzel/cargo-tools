use futures::channel::mpsc::{self, UnboundedSender};
use iced_runtime::Action;

pub struct EventLoop<T> {
    rx: mpsc::UnboundedReceiver<Action<T>>,
}

impl<T> EventLoop<T> {
    pub fn new() -> (UnboundedSender<Action<T>>, Self) {
        let (tx, rx) = mpsc::unbounded();
        (tx, Self { rx })
    }

    pub fn run(mut self, mut f: impl FnMut(T)) {
        loop {
            match self.rx.try_next() {
                Ok(message) => match message {
                    Some(action) => match action {
                        Action::Output(message) => f(message),
                        Action::Exit => break,
                        _ => continue,
                    },
                    None => break,
                },
                Err(_) => continue,
            }
        }
    }
}
