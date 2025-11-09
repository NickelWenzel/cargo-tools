use cargo_tools::state_manager::{StateManager, StateValue};
use cargo_tools_macros::wasm_async_trait;
use serde_wasm_bindgen::{from_value, to_value};
use thiserror::Error;

use crate::vs_code_api::{get_state, update_state, JsValueExt};

/// Error type for VSCodeStateManager operations
#[derive(Error, Debug)]
pub enum StateManagerError {
    /// Error during serialization/deserialization with serde_wasm_bindgen
    #[error("Serialization error")]
    SerializationError(#[source] serde_wasm_bindgen::Error),

    /// Error during state update operation from TypeScript
    #[error("State update error")]
    UpdateError(String),
}

pub struct VSCodeStateManager;

#[wasm_async_trait]
impl StateManager for VSCodeStateManager {
    type UpdateError = StateManagerError;

    fn get<T: StateValue>(&self) -> Option<T> {
        get_state(T::KEY).and_then(|v| from_value(v).ok())
    }

    async fn update<T: StateValue>(&self, value: T) -> Result<(), Self::UpdateError> {
        let value = to_value(&value).map_err(StateManagerError::SerializationError)?;
        update_state(T::KEY.to_string(), value)
            .await
            .map_err(|e| StateManagerError::UpdateError(e.as_error_string()))?;
        Ok(())
    }

    fn add_on_changed_handler<T: StateValue>(&self, _changed_handler: impl AsyncFn()) {
        todo!()
    }
    fn reset_changed_handlers() {
        todo!()
    }
}
