//! A default, cross-platform backend.
//!
//! - On native platforms, it will use:
//!   - `iced_headless::tokio_context::TokioContext` when the `tokio` feature is enabled.
//!   - `iced_futures::backend::default::Executor` otherwise`
#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[cfg(feature = "tokio")]
    pub use crate::tokio_context::*;

    #[cfg(not(feature = "tokio"))]
    pub use iced_futures::backend::default::*;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use iced_futures::backend::default::*;
}

pub use platform::*;
