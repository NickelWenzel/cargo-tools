//! HeadlessProgram trait for headless applications.

use cargo_tools_macros::wasm_async_trait;
use iced::Task;
use iced_futures::{subscription, Executor, MaybeSend, Runtime, Subscription};
use iced_runtime::Action;

use crate::{
    event_loop::{EventLoop, Exit},
    Error, Result,
};

/// A headless application with no UI.
///
/// This trait defines the lifecycle and behavior of a headless application,
/// similar to iced's `Program` trait but without rendering, themes, or windows.
///
/// State is managed externally by the runtime, matching iced 0.13.1's approach.
#[wasm_async_trait]
pub trait HeadlessProgram: Sized {
    /// The state maintained by the program.
    type State;

    /// The type of messages handled by the program.
    type Message: Send + std::fmt::Debug + 'static;

    /// The executor used to spawn asynchronous tasks.
    type Executor: Executor + MaybeSend;

    /// Updates the program state in response to a message.
    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message>;

    /// Returns the subscriptions for the program.
    ///
    /// Subscriptions are streams of events that produce messages.
    /// The program runs until all subscriptions complete.
    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn exit_on(&self, _state: &Self::State) -> Subscription<Exit> {
        Subscription::none()
    }

    /// Runs the [`HeadlessProgram`].
    ///
    /// The state of the [`HeadlessProgram`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    async fn run(self) -> Result<()>
    where
        Self: 'static,
        Self::State: MaybeSend + Default,
        Self::Executor: MaybeSend,
    {
        self.run_with(|| (Self::State::default(), Task::none()))
            .await
    }

    /// Runs the [`HeadlessProgram`] with the given [`Settings`] and a closure that creates the initial state.
    async fn run_with<I>(self, initialize: I) -> Result<()>
    where
        Self: 'static,
        Self::State: MaybeSend,
        Self::Executor: MaybeSend,
        I: FnOnce() -> (Self::State, Task<Self::Message>) + MaybeSend + 'static,
    {
        let (to_event_loop_tx, event_loop) = EventLoop::new();

        let mut runtime: Runtime<
            <Self as HeadlessProgram>::Executor,
            futures::channel::mpsc::UnboundedSender<Action<<Self as HeadlessProgram>::Message>>,
            Action<<Self as HeadlessProgram>::Message>,
        > = {
            let executor = Self::Executor::new().map_err(Error::ExecutorCreationFailed)?;

            Runtime::new(executor, to_event_loop_tx)
        };

        let (state, task) = runtime.enter(initialize);

        if let Some(stream) = iced_runtime::task::into_stream(task) {
            runtime.run(stream);
        }

        runtime.track(subscription::into_recipes(
            runtime.enter(|| self.subscription(&state).map(Action::Output)),
        ));

        runtime.track(subscription::into_recipes(
            runtime.enter(|| self.exit_on(&state).map(|_| Action::Exit)),
        ));

        event_loop
            .run(state, move |state, message| {
                let task = self.update(state, message);

                if let Some(stream) = iced_runtime::task::into_stream(task) {
                    runtime.run(stream);
                }

                runtime.track(subscription::into_recipes(
                    runtime.enter(|| self.subscription(state).map(Action::Output)),
                ));

                runtime.track(subscription::into_recipes(
                    runtime.enter(|| self.exit_on(state).map(|_| Action::Exit)),
                ));
            })
            .await;

        Ok(())
    }
}
