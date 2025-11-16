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
//!         .run()
//!         .await
//! }
//! ```
//!
//! ## With Timeout (requires `tokio` feature)
//!
//! ```ignore
//! use std::time::Duration;
//!
//! viewless::<MyProgram>()
//!     .timeout(Duration::from_secs(10))
//!     .run()
//!     .await?;
//! ```
//!
//! ## Custom Executor
//!
//! ```ignore
//! let executor = iced_futures::backend::tokio::Executor::new()?;
//!
//! viewless::<MyProgram>()
//!     .run_with_executor(executor)
//!     .await?;
//! ```

pub mod error;
pub mod event;
pub mod program;
pub mod runtime;
pub mod viewless;

pub use error::{Error, Result};
pub use program::{Instance, ViewlessProgram};
pub use viewless::{viewless, Viewless};

pub use iced_futures::{Executor, Subscription};
