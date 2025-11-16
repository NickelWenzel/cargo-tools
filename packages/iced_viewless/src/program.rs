//! Program trait and instance management for viewless applications.

use iced_futures::{Executor, Subscription};

/// A headless application with no UI.
///
/// This trait defines the lifecycle and behavior of a viewless application,
/// similar to iced's `Program` trait but without rendering, themes, or windows.
pub trait ViewlessProgram: Sized {
    /// The state maintained by the program.
    type State;

    /// The type of messages handled by the program.
    type Message: Send + 'static;

    /// The executor used to spawn asynchronous tasks.
    type Executor: Executor;

    /// Returns the unique name of the program.
    fn name() -> &'static str;

    /// Initializes the program and returns the initial state.
    fn boot(&self) -> Self::State;

    /// Updates the program state in response to a message.
    fn update(&self, state: &mut Self::State, message: Self::Message);

    /// Returns the subscriptions for the program.
    ///
    /// Subscriptions are streams of events that produce messages.
    /// The program runs until all subscriptions complete.
    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        Subscription::none()
    }
}

/// A running instance of a viewless program.
///
/// This wraps the program and its current state, managing the lifecycle
/// following iced's `program::Instance` pattern.
#[derive(Debug)]
pub struct Instance<P: ViewlessProgram> {
    program: P,
    state: P::State,
}

impl<P: ViewlessProgram> Instance<P> {
    /// Creates a new program instance by booting the program.
    pub fn new(program: P) -> Self {
        let state = program.boot();
        Self { program, state }
    }

    /// Updates the program state with a message.
    pub fn update(&mut self, message: P::Message) {
        self.program.update(&mut self.state, message)
    }

    /// Returns the current subscription for the program.
    pub fn subscription(&self) -> Subscription<P::Message> {
        self.program.subscription(&self.state)
    }

    /// Returns a reference to the program.
    pub fn program(&self) -> &P {
        &self.program
    }

    /// Returns a reference to the current state.
    pub fn state(&self) -> &P::State {
        &self.state
    }
}
