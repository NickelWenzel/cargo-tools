//! A default, cross-platform backend.
//!
//! - On native platforms, it will use:
//!   - `iced_viewless::tokio_context::TokioContext` when the `tokio` feature is enabled.
//!   - `iced_futures::backend::default::Executor` otherwise`
#[cfg(not(target_arch = "wasm32"))]
mod platform {
    pub(crate) use futures::executor::block_on;

    pub mod async_context {
        // Avoid enetering tokio runtime twice in async contexts
        #[cfg(feature = "tokio")]
        pub use crate::tokio_context::*;

        #[cfg(not(feature = "tokio"))]
        pub use iced_futures::backend::default::*;
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub(crate) use wasm_bindgen_futures::spawn_local as block_on;

    pub mod async_context {
        pub use iced_futures::backend::default::*;
    }
}

pub use iced_futures::backend::default::*;
pub(crate) use platform::*;
