//! A viewless runtime for iced applications.
//!
//! `iced_viewless` provides a viewless runtime for iced applications that don't require
//! windowing and rendering:
//!
//! - Integration in other UI frameworks
//! - Background services with subscription-based event handling
//! - CLI tools with async operations
//! - Testing application logic without UI overhead
//! - Embedded systems without display capabilities
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use futures::SinkExt;
//! use iced_viewless::{Subscription, Task, application, event_loop::Exit};
//!
//! #[derive(Default)]
//! struct MyState { done: bool }
//!
//! #[derive(Debug, Clone)]
//! enum Message { Tick }
//!
//! fn update(state: &mut MyState, message: Message) -> Task<Message> {
//!     match message {
//!         Message::Tick => state.done = true,
//!     }
//!     Task::none()
//! }
//!
//! fn exit_on(state: &MyState) -> Subscription<Exit> {
//!     if state.done {
//!         Subscription::run(|| iced_viewless::stream::channel(1, |mut tx: futures::channel::mpsc::Sender<Exit>| async move {
//!             let _ = tx.send(Exit).await;
//!         }))
//!     } else {
//!         Subscription::none()
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     application(update)
//!         .exit_on(exit_on)
//!         .run()?;
//!     Ok(())
//! }
//! ```
//!
//! ## With Initial State and Startup Task
//!
//! ```no_run
//! use futures::SinkExt;
//! use iced_viewless::{Subscription, Task, application, event_loop::Exit};
//!
//! #[derive(Default)]
//! struct MyState { done: bool }
//!
//! #[derive(Debug, Clone)]
//! enum Message { Tick }
//!
//! fn update(state: &mut MyState, message: Message) -> Task<Message> {
//!     match message {
//!         Message::Tick => state.done = true,
//!     }
//!     Task::none()
//! }
//!
//! fn exit_on(state: &MyState) -> Subscription<Exit> {
//!     if state.done { Subscription::none() } else { Subscription::none() }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     application(update)
//!         .exit_on(exit_on)
//!         .run_with(|| (MyState { done: false }, Task::done(Message::Tick)))?;
//!     Ok(())
//! }
//! ```
//!
//! ## Within an Async Context
//!
//! ```no_run
//! use futures::SinkExt;
//! use iced_viewless::{Subscription, Task, event_loop::Exit, viewless::async_application};
//!
//! #[derive(Default)]
//! struct MyState { done: bool }
//!
//! #[derive(Debug, Clone)]
//! enum Message { Tick }
//!
//! fn update(state: &mut MyState, message: Message) -> Task<Message> {
//!     match message {
//!         Message::Tick => state.done = true,
//!     }
//!     Task::none()
//! }
//!
//! fn exit_on(state: &MyState) -> Subscription<Exit> {
//!     if state.done {
//!         Subscription::run(|| iced_viewless::stream::channel(1, |mut tx: futures::channel::mpsc::Sender<Exit>| async move {
//!             let _ = tx.send(Exit).await;
//!         }))
//!     } else {
//!         Subscription::none()
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // In a real application you would .await this directly inside an
//!     // existing async context (e.g. inside a tokio task).
//!     futures::executor::block_on(async {
//!         async_application(update)
//!             .exit_on(exit_on)
//!             .run()
//!             .await
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! ## With Custom Executor
//!
//! ```no_run
//! use iced_viewless::{Subscription, Task, application, event_loop::Exit};
//!
//! #[derive(Default)]
//! struct MyState;
//!
//! #[derive(Debug, Clone)]
//! enum Message {}
//!
//! fn update(_state: &mut MyState, message: Message) -> Task<Message> {
//!     Task::none()
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     application(update)
//!         .executor::<iced_futures::backend::default::Executor>()
//!         .run()?;
//!     Ok(())
//! }
//! ```

pub mod default;
pub mod error;
pub mod event_loop;
pub mod program;
#[cfg(feature = "tokio")]
pub mod tokio_context;
pub mod viewless;

pub use error::{Error, Result};
pub use program::ViewlessProgram;
pub use viewless::{Application, WithExecutor, WithSubscription, application, async_application};

pub use iced_futures::{Subscription, stream};
pub use iced_runtime::Task;
