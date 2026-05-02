//! Builder API for creating and running viewless applications.

pub use std::convert::Infallible as Never;

use crate::{Result, default};
use crate::{event_loop::Exit, program::ViewlessProgram};
use iced_futures::{Executor, MaybeSend, Subscription};
use iced_runtime::Task;

/// A builder for viewless applications implementing iced's Program trait.
///
/// # Examples
/// ```ignore
/// use iced_viewless::application;
/// use iced_core::Program;
///
/// let app = application(my_program)
///     .subscription(|state| my_subscription(state))
///     .executor::<MyExecutor>();
///
/// app.run(|| MyState::default()).await?;
/// ```
pub struct Application<P> {
    raw: P,
}

impl<P: ViewlessProgram> Application<P>
where
    Self: 'static,
{
    /// Runs the [`Application`].
    ///
    /// The state of the [`Application`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    pub fn run(self) -> Result<()>
    where
        P: MaybeSend,
        P::State: Default + MaybeSend,
        P::Executor: MaybeSend,
    {
        default::block_on(self.raw.run()?);
        Ok(())
    }

    /// Runs the [`Application`] with a closure that creates the initial state.
    pub fn run_with<I>(self, initialize: I) -> Result<()>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        I: FnOnce() -> (P::State, Task<P::Message>) + MaybeSend + 'static,
    {
        default::block_on(self.raw.run_with(initialize)?);
        Ok(())
    }

    /// Sets the subscription logic of the [`Application`].
    pub fn subscription<F>(self, f: F) -> Application<WithSubscription<P, F>>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        F: Fn(&P::State) -> Subscription<P::Message>,
    {
        Application {
            raw: WithSubscription {
                program: self.raw,
                subscription: f,
            },
        }
    }

    pub fn exit_on<F>(self, f: F) -> Application<WithExitOn<P, F>>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        F: Fn(&P::State) -> Subscription<Exit>,
    {
        Application {
            raw: WithExitOn {
                program: self.raw,
                exit_on: f,
            },
        }
    }

    /// Sets the executor of the [`Application`].
    pub fn executor<E>(self) -> Application<WithExecutor<P, E>>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        E: Executor + MaybeSend,
    {
        Application {
            raw: WithExecutor {
                program: self.raw,
                _executor: std::marker::PhantomData,
            },
        }
    }
}

/// A builder for viewless async applications which are run within an async context implementing iced's Program trait.
///
/// This provides a fluent API similar to `iced::Application` but for viewless execution.
/// Follows iced's decorator pattern with `raw` field storing the program implementation.
///
/// # Examples
/// ```ignore
/// use iced_viewless::application;
/// use iced_core::Program;
///
/// let app = application(my_program)
///     .subscription(|state| my_subscription(state))
///     .executor::<MyExecutor>();
///
/// app.run(|| MyState::default()).await?;
/// ```
pub struct AsyncApplication<P>(Application<P>);

impl<P: ViewlessProgram> AsyncApplication<P>
where
    Self: 'static,
{
    /// Runs the [`AsyncApplication`].
    ///
    /// The state of the [`AsyncApplication`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    pub async fn run(self) -> Result<()>
    where
        P: MaybeSend,
        P::State: Default + MaybeSend,
        P::Executor: MaybeSend,
    {
        self.0.raw.run()?.await;
        Ok(())
    }

    /// Runs the [`AsyncApplication`] with a closure that creates the initial state.
    pub async fn run_with<I>(self, initialize: I) -> Result<()>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        I: FnOnce() -> (P::State, Task<P::Message>) + MaybeSend + 'static,
    {
        self.0.raw.run_with(initialize)?.await;
        Ok(())
    }

    /// Sets the subscription logic of the [`AsyncApplication`].
    pub fn subscription<F>(self, f: F) -> AsyncApplication<WithSubscription<P, F>>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        F: Fn(&P::State) -> Subscription<P::Message>,
    {
        AsyncApplication(self.0.subscription(f))
    }

    pub fn exit_on<F>(self, f: F) -> AsyncApplication<WithExitOn<P, F>>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        F: Fn(&P::State) -> Subscription<Exit>,
    {
        AsyncApplication(self.0.exit_on(f))
    }

    /// Sets the executor of the [`AsyncApplication`].
    pub fn executor<E>(self) -> AsyncApplication<WithExecutor<P, E>>
    where
        P: MaybeSend,
        P::State: MaybeSend,
        P::Executor: MaybeSend,
        E: Executor + MaybeSend,
    {
        AsyncApplication(self.0.executor())
    }
}

/// Decorator that adds a custom subscription function to a program.
///
/// Follows iced's `program::with_subscription` pattern.
pub struct WithSubscription<P, F> {
    program: P,
    subscription: F,
}

impl<P, F> ViewlessProgram for WithSubscription<P, F>
where
    P: ViewlessProgram,
    F: Fn(&P::State) -> Subscription<P::Message>,
{
    type State = P::State;
    type Message = P::Message;
    type Executor = P::Executor;

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        self.program.update(state, message)
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        (self.subscription)(state)
    }

    fn exit_on(&self, state: &Self::State) -> Subscription<Exit> {
        self.program.exit_on(state)
    }
}

/// Decorator that adds a custom subscription function to a program.
///
/// Follows iced's `program::with_subscription` pattern.
pub struct WithExitOn<P, F> {
    program: P,
    exit_on: F,
}

impl<P, F> ViewlessProgram for WithExitOn<P, F>
where
    P: ViewlessProgram,
    F: Fn(&P::State) -> Subscription<Exit>,
{
    type State = P::State;
    type Message = P::Message;
    type Executor = P::Executor;

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        self.program.update(state, message)
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        self.program.subscription(state)
    }

    fn exit_on(&self, state: &Self::State) -> Subscription<Exit> {
        (self.exit_on)(state)
    }
}

/// Decorator that changes the executor type of a program.
///
/// Follows iced's decorator pattern for executor customization.
pub struct WithExecutor<P, E> {
    program: P,
    _executor: std::marker::PhantomData<E>,
}

impl<P, E> ViewlessProgram for WithExecutor<P, E>
where
    P: ViewlessProgram,
    E: Executor + MaybeSend,
{
    type State = P::State;
    type Message = P::Message;
    type Executor = E;

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        self.program.update(state, message)
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        self.program.subscription(state)
    }

    fn exit_on(&self, state: &Self::State) -> Subscription<Exit> {
        self.program.exit_on(state)
    }
}

/// Creates a new viewless application.
///
/// This is the primary entry point for creating viewless applications,
/// matching iced's `application()` function pattern.
///
/// # Arguments
/// * `program` - A type implementing `ViewlessProgram`
///
/// # Examples
/// ```ignore
/// use iced_viewless::{application, ViewlessProgram};
///
/// #[derive(Default)]
/// struct MyProgram;
///
/// impl ViewlessProgram for MyProgram {
///     // ... implementation ...
/// }
///
/// application(MyProgram::default())
///     .run(|| ())?;
/// ```
pub fn application<State, Message>(
    update: impl UpdateFn<State, Message>,
) -> Application<impl ViewlessProgram<State = State, Message = Message>>
where
    State: 'static,
    Message: Send + std::fmt::Debug + 'static,
{
    use std::marker::PhantomData;

    struct Instance<State, Message, Update> {
        update: Update,
        _state: PhantomData<State>,
        _message: PhantomData<Message>,
    }

    impl<State, Message, Update> ViewlessProgram for Instance<State, Message, Update>
    where
        Message: Send + std::fmt::Debug + 'static,
        Update: self::UpdateFn<State, Message>,
    {
        type State = State;
        type Message = Message;
        type Executor = default::Executor;

        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.update.update(state, message)
        }
    }

    Application {
        raw: Instance {
            update,
            _state: PhantomData,
            _message: PhantomData,
        },
    }
}

/// Creates a new viewless async application.
///
/// This is the primary entry point for creating viewless applications,
/// matching iced's `application()` function pattern.
///
/// # Arguments
/// * `program` - A type implementing `ViewlessProgram`
///
/// # Examples
/// ```ignore
/// use iced_viewless::{application, ViewlessProgram};
///
/// #[derive(Default)]
/// struct MyProgram;
///
/// impl ViewlessProgram for MyProgram {
///     // ... implementation ...
/// }
///
/// async_application(MyProgram::default())
///     .run(|| ())
///     .await?;
/// ```
pub fn async_application<State, Message>(
    update: impl UpdateFn<State, Message> + 'static + MaybeSend,
) -> AsyncApplication<impl ViewlessProgram<State = State, Message = Message>>
where
    State: 'static + MaybeSend,
    Message: Send + std::fmt::Debug + 'static,
{
    AsyncApplication(application(update)).executor::<default::async_context::Executor>()
}

/// The update logic of some [`Application`].
///
/// This trait allows the [`application`] builder to take any closure that
/// returns any `Into<Task<Message>>`.
///
/// This trait is copied directly from [Iced](https://github.com/iced-rs/iced/blob/0.14/src/application.rs#L598).
pub trait UpdateFn<State, Message> {
    /// Processes the message and updates the state of the [`Application`].
    fn update(&self, state: &mut State, message: Message) -> Task<Message>;
}

impl<State> UpdateFn<State, Never> for () {
    fn update(&self, _state: &mut State, _message: Never) -> Task<Never> {
        Task::none()
    }
}

impl<T, State, Message, C> UpdateFn<State, Message> for T
where
    T: Fn(&mut State, Message) -> C,
    C: Into<Task<Message>>,
{
    fn update(&self, state: &mut State, message: Message) -> Task<Message> {
        self(state, message).into()
    }
}
