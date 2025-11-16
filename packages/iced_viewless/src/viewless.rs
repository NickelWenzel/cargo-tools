//! Builder API for creating and running viewless applications.

use crate::program::{Instance, ViewlessProgram};
use crate::Result;
use iced_futures::{Executor, Subscription};
use std::time::Duration;

/// A builder for viewless applications.
///
/// This provides a fluent API for configuring and running a viewless application,
/// similar to `iced::daemon()` but without windowing or rendering.
///
/// # Examples
/// ```ignore
/// use iced_viewless::{viewless, ViewlessProgram};
///
/// viewless::<MyProgram>()
///     .settings(Settings::default())
///     .run()
///     .await?;
/// ```
pub struct Viewless<P>
where
    P: ViewlessProgram,
{
    timeout: Option<Duration>,
    _program: std::marker::PhantomData<P>,
}

impl<P> Viewless<P>
where
    P: ViewlessProgram,
{
    /// Creates a new viewless application builder.
    pub fn new() -> Self {
        Self {
            timeout: None,
            _program: std::marker::PhantomData,
        }
    }

    /// Sets an optional timeout for the application.
    ///
    /// If set, the application will automatically exit after the specified duration,
    /// even if subscriptions are still active.
    ///
    /// # Arguments
    /// * `timeout` - Maximum runtime duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Runs the viewless application with the default executor.
    ///
    /// This method:
    /// 1. Creates an executor
    /// 2. Boots the program
    /// 3. Runs until all subscriptions complete
    ///
    /// # Returns
    /// `Ok(())` when the application completes normally, or an error if startup or runtime fails.
    pub async fn run(self) -> Result<()>
    where
        P: Default,
        P::Executor: Default,
    {
        let executor = P::Executor::default();
        self.run_with_executor(executor).await
    }

    /// Runs the viewless application with the provided executor.
    ///
    /// This method:
    /// 1. Uses the provided executor
    /// 2. Boots the program
    /// 3. Runs until all subscriptions complete or timeout is reached
    ///
    /// # Arguments
    /// * `executor` - The executor to use for running subscriptions
    ///
    /// # Returns
    /// `Ok(())` when the application completes normally, or an error if startup or runtime fails.
    pub async fn run_with_executor(self, executor: P::Executor) -> Result<()>
    where
        P: Default,
    {
        let program = P::default();
        let instance = Instance::new(program);

        #[cfg(feature = "tokio")]
        if let Some(timeout) = self.timeout {
            let run_future = crate::runtime::run(executor, instance);
            return tokio::select! {
                result = run_future => result,
                _ = tokio::time::sleep(timeout) => Ok(()),
            };
        }

        crate::runtime::run(executor, instance).await
    }

    /// Runs a subscription until it completes.
    ///
    /// This is a convenience method for running a single subscription
    /// without needing to implement a full ViewlessProgram.
    ///
    /// # Arguments
    /// * `subscription` - The subscription to track
    ///
    /// # Returns
    /// `Ok(())` when the subscription completes.
    pub async fn run_subscription<Message>(
        subscription: impl Fn() -> Subscription<Message> + 'static,
    ) -> Result<()>
    where
        Message: 'static + Send,
    {
        struct SubscriptionProgram<S> {
            subscription: S,
        }

        impl<S, M> ViewlessProgram for SubscriptionProgram<S>
        where
            M: 'static + Send,
            S: Fn() -> Subscription<M> + 'static,
        {
            type State = ();
            type Message = M;
            type Executor = iced_futures::backend::default::Executor;

            fn name() -> &'static str {
                "Viewless Subscription"
            }

            fn boot(&self) -> Self::State {}

            fn update(&self, _state: &mut Self::State, _message: Self::Message) {}

            fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
                (self.subscription)()
            }
        }

        let program = SubscriptionProgram { subscription };

        let instance = Instance::new(program);
        let executor = match iced_futures::backend::default::Executor::new() {
            Ok(exec) => exec,
            Err(e) => return Err(crate::error::Error::ExecutorCreationFailed(e)),
        };
        crate::runtime::run(executor, instance).await
    }
}

impl<P> Default for Viewless<P>
where
    P: ViewlessProgram,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new viewless application builder.
///
/// This is the primary entry point for creating viewless applications.
///
/// # Examples
/// ```ignore
/// use iced_viewless::{viewless, ViewlessProgram};
///
/// #[derive(Default)]
/// struct MyProgram;
///
/// impl ViewlessProgram for MyProgram {
///     // ... implementation ...
/// }
///
/// viewless::<MyProgram>()
///     .run()
///     .await?;
/// ```
pub fn viewless<P>() -> Viewless<P>
where
    P: ViewlessProgram,
{
    Viewless::new()
}
