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

    fn block_on<T>(&self, _future: impl Future<Output = T>) -> T {
        panic!(
            "This executor should run within an existing tokio context and therefore you should never call block_on."
        )
    }
}
