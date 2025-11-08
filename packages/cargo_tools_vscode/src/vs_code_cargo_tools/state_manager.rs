use async_trait::async_trait;
use serde_wasm_bindgen::{from_value, to_value};

use crate::{
    state_manager::{StateManager, StateValue},
    vs_code_api::StateManagerTS,
};

pub struct VSCodeStateManager(StateManagerTS);

impl VSCodeStateManager {
    pub fn new(state_manager_inner: StateManagerTS) -> Self {
        Self(state_manager_inner)
    }
}

#[async_trait(?Send)]
impl StateManager for VSCodeStateManager {
    type UpdateError = serde_wasm_bindgen::Error;

    fn get<T: StateValue>(&self) -> Option<T> {
        self.0.get(T::KEY).and_then(|v| from_value(v).ok())
    }

    async fn update<T: StateValue + Send + Sync + 'static>(
        &self,
        value: T,
    ) -> Result<(), Self::UpdateError> {
        let value = to_value(&value)?;
        self.0.update(T::KEY.to_string(), value).await;
        Ok(())
    }
}
