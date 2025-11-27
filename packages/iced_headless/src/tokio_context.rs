use std::future::Future;

use iced::Executor;
use iced_futures::MaybeSend;

pub struct TokioContext;

impl Executor for TokioContext {
    fn new() -> Result<Self, futures::io::Error>
    where
        Self: Sized,
    {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + MaybeSend + 'static) {
        tokio::spawn(future);
    }
}
