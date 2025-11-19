//! A headless runtime for iced applications.
//!
//! `iced_viewless` provides a viewless (headless) runtime for iced applications that don't require
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
//! - [`ViewlessProgram`] - Core trait defining application lifecycle (similar to `iced::Program`)
//! - [`viewless()`] - Builder API for creating and running applications
//!
//! # Features
//!
//! The crate re-exports executor features from `iced_futures`:
//!
//! - `tokio` - Use Tokio runtime (enables timeout support)
//! - `async-std` - Use async-std runtime
//! - `smol` - Use smol runtime
//! - `thread-pool` - Use thread pool executor
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```ignore
//! use iced_viewless::{viewless, ViewlessProgram, Subscription, Executor};
//!
//! #[derive(Default)]
//! struct MyProgram;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Tick,
//! }
//!
//! impl ViewlessProgram for MyProgram {
//!     type State = ();
//!     type Message = Message;
//!     type Executor = iced_futures::backend::default::Executor;
//!
//!     fn name() -> &'static str {
//!         "My Viewless App"
//!     }
//!
//!     fn boot(&self) -> Self::State {
//!         ()
//!     }
//!
//!     fn update(&self, _state: &mut Self::State, message: Self::Message) {
//!         match message {
//!             Message::Tick => println!("Tick!"),
//!         }
//!     }
//!
//!     fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
//!         use iced_futures::subscription;
//!         use std::time::Duration;
//!         
//!         subscription::unfold(
//!             0,
//!             Duration::from_secs(1),
//!             |state| async move {
//!                 tokio::time::sleep(Duration::from_secs(1)).await;
//!                 Some((Message::Tick, state + 1))
//!             }
//!         )
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> iced_viewless::Result<()> {
//!     viewless::<MyProgram>()
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```ignore
//! use iced_viewless::{application, ViewlessProgram, Subscription};
//!
//! #[derive(Default)]
//! struct MyProgram;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Tick,
//! }
//!
//! impl ViewlessProgram for MyProgram {
//!     type State = ();
//!     type Message = Message;
//!     type Executor = iced_futures::backend::default::Executor;
//!
//!     fn name() -> &'static str {
//!         "My Viewless App"
//!     }
//!
//!     fn boot(&self) -> Self::State {
//!         ()
//!     }
//!
//!     fn update(&self, _state: &mut Self::State, _message: Self::Message) {
//!         // Handle messages
//!     }
//!
//!     fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
//!         // Return subscriptions
//!         Subscription::none()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     application(MyProgram::default())
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

pub mod error;
pub mod event_loop;
pub mod program;
#[cfg(feature = "tokio")]
pub mod tokio_context;
pub mod viewless;

pub use error::{Error, Result};
pub use program::ViewlessProgram;
pub use viewless::{application, Application, WithExecutor, WithSubscription};

pub use iced_futures::{Executor, Subscription};
