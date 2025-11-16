//! Builder API for creating and running viewless applications.

use crate::program::ViewlessProgram;
use crate::Result;
use iced::{application::Update, Task};
use iced_futures::{Executor, Subscription};

/// A builder for viewless applications implementing iced's Program trait.
///
/// This provides a fluent API similar to `iced::Application` but for headless execution.
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
pub struct Application<P> {
    raw: P,
}

impl<P: ViewlessProgram> Application<P>
where
    Self: 'static,
    P::State: Default,
{
    /// Runs the [`Application`].
    ///
    /// The state of the [`Application`] must implement [`Default`].
    /// If your state does not implement [`Default`], use [`run_with`]
    /// instead.
    ///
    /// [`run_with`]: Self::run_with
    pub async fn run(self) -> Result<()> {
        self.raw.run()
    }

    /// Runs the [`Application`] with a closure that creates the initial state.
    pub fn run_with<I>(self, initialize: I) -> Result<()>
    where
        Self: 'static,
        I: FnOnce() -> (P::State, Task<P::Message>) + 'static,
    {
        self.raw.run_with(initialize)
    }

    /// Sets the subscription logic of the [`Application`].
    pub fn subscription<F>(self, f: F) -> Application<WithSubscription<P, F>>
    where
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
        F: Fn(&P::State) -> Subscription<()>,
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
        E: Executor,
    {
        Application {
            raw: WithExecutor {
                program: self.raw,
                _executor: std::marker::PhantomData,
            },
        }
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

    fn exit_on(&self) -> Subscription<()> {
        self.program.exit_on()
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
    F: Fn() -> Subscription<()>,
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

    fn exit_on(&self) -> Subscription<()> {
        (self.exit_on)()
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
    E: Executor,
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

    fn exit_on(&self) -> Subscription<()> {
        self.program.exit_on()
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
///     .run(|| ())
///     .await?;
/// ```
pub fn application<State, Message>(
    update: impl Update<State, Message>,
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
        Update: self::Update<State, Message>,
    {
        type State = State;
        type Message = Message;
        type Executor = iced_futures::backend::default::Executor;

        fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
            self.update.update(state, message).into()
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
