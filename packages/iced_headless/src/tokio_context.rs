use std::future::Future;

use iced_futures::MaybeSend;

pub struct Executor;

impl iced_futures::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error>
    where
        Self: Sized,
    {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + MaybeSend + 'static) {
        tokio::task::spawn(future);
    }
}
