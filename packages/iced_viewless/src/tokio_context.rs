use std::future::Future;

use iced_futures::MaybeSend;

/// An iced executor that spawns futures onto an existing tokio runtime via
/// [`tokio::task::spawn`].
///
/// Used by [`async_application`](crate::viewless::async_application) when the `tokio`
/// feature is enabled, so that the viewless runtime integrates with a pre-existing
/// tokio context rather than creating a new one. Calling [`block_on`](iced_futures::Executor::block_on)
/// on this executor will panic — use it only from within a running tokio runtime.
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
