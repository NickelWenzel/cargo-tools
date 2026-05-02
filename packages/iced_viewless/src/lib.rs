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
pub mod program;
#[cfg(feature = "tokio")]
pub mod tokio_context;
pub mod viewless;

pub use error::{Error, Result};
pub use program::ViewlessProgram;
pub use viewless::{Application, WithExecutor, WithSubscription, application, async_application};

pub use iced_futures::{Subscription, stream};
pub use iced_runtime::Task;
