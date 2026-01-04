//! A headless runtime for iced applications.
//!
//! `iced_headless` provides a headless runtime for iced applications that don't require
//! windowing, rendering, or user interaction. It replicates iced's program functionality
//! while excluding UI-specific code, making it ideal for:
//!
//! - Background services with subscription-based event handling
//! - CLI tools with async operations
//! - Testing application logic without UI overhead
//! - Embedded systems without display capabilities
//!
//! # Architecture
//!
//! The crate follows iced's program pattern:
//!
//! - [`HeadlessProgram`] - Core trait defining application lifecycle (similar to `iced::Program`)
//! - [`application()`] - Builder API for creating and running applications
//!
//! # Features
//!
//! Compatible with iced 0.13.1.
//!
//! The crate re-exports executor features from `iced_futures`:
//!
//! - `tokio` - Use Tokio runtime
//! - `async-std` - Use async-std runtime
//! - `smol` - Use smol runtime
//! - `thread-pool` - Use thread pool executor
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```ignore
//! use iced_headless::{application, HeadlessProgram, Subscription};
//!
//! #[derive(Default)]
//! struct MyProgram;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Tick,
//! }
//!
//! impl HeadlessProgram for MyProgram {
//!     type State = ();
//!     type Message = Message;
//!     type Executor = iced_futures::backend::default::Executor;
//!
//!     fn update(&self, _state: &mut Self::State, _message: Self::Message) -> iced::Task<Self::Message> {
//!         iced::Task::none()
//!     }
//!
//!     fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
//!         Subscription::none()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     application(MyProgram::update)
//!         .run(|| ())
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! ## With Custom Subscription
//!
//! ```ignore
//! application(MyProgram::default())
//!     .subscription(|state| my_custom_subscription(state))
//!     .run(|| MyState::default())
//!     .await?;
//! ```
//!
//! ## With Custom Executor
//!
//! ```ignore
//! application(MyProgram::default())
//!     .executor::<iced_futures::backend::tokio::Executor>()
//!     .run(|| MyState::default())
//!     .await?;
//! ```

pub mod default;
pub mod error;
pub mod event_loop;
pub mod headless;
pub mod program;
#[cfg(feature = "tokio")]
pub mod tokio_context;

pub use error::{Error, Result};
pub use headless::{Application, WithExecutor, WithSubscription, application, async_application};
pub use program::HeadlessProgram;

pub use iced::{Subscription, Task, stream};
