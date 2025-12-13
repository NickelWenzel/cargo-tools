use cargo_tools::state::{Callback, ConfigurationManager, State, StateValue};
use serde_wasm_bindgen::{from_value, to_value};
use thiserror::Error;
use wasm_async_trait::wasm_async_trait;
use wasm_bindgen::JsValue;

use crate::vs_code_api::{get_state, update_state, JsValueExt};

/// Error type for VSCodeStateManager operations
#[derive(Error, Debug)]
pub enum StateManagerError {
    /// Error during serialization/deserialization with serde_wasm_bindgen
    #[error("Serialization error")]
    SerializationError(#[source] serde_wasm_bindgen::Error),

    /// Error during state update operation from TypeScript
    #[error("State update error: {0}")]
    UpdateError(String),
}

pub struct VSCodeStateManager {
    subscriptions: Vec<Box<dyn Callback>>,
}

impl VSCodeStateManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    async fn call_subscriptions(&self) {
        for sub in &self.subscriptions {
            sub.call(&self.get_state()).await;
        }
    }

    fn get_state(&self) -> State {
        use cargo_tools::state::*;

        State {
            selected_package: get_state(SelectedPackage::KEY).and_then(|v| from_value(v).ok()),
            selected_build_target: get_state(SelectedBuildTarget::KEY)
                .and_then(|v| from_value(v).ok()),
            selected_run_target: get_state(SelectedRunTarget::KEY).and_then(|v| from_value(v).ok()),
            selected_benchmark_target: get_state(SelectedBenchmarkTarget::KEY)
                .and_then(|v| from_value(v).ok()),
            selected_platform_target: get_state(SelectedPlatformTarget::KEY)
                .and_then(|v| from_value(v).ok()),
            selected_features: get_state(SelectedFeatures::KEY).and_then(|v| from_value(v).ok()),
            selected_profile: get_state(SelectedProfile::KEY).and_then(|v| from_value(v).ok()),
            group_by_workspace_member: get_state(GroupByWorkspaceMember::KEY)
                .and_then(|v| from_value(v).ok()),
            workspace_member_filter: get_state(WorkspaceMemberFilter::KEY)
                .and_then(|v| from_value(v).ok()),
            target_type_filter: get_state(TargetTypeFilter::KEY).and_then(|v| from_value(v).ok()),
            is_target_type_filter_active: get_state(IsTargetTypeFilterActive::KEY)
                .and_then(|v| from_value(v).ok()),
            show_features: get_state(ShowFeatures::KEY).and_then(|v| from_value(v).ok()),
            makefile_task_filter: get_state(MakefileTaskFilter::KEY)
                .and_then(|v| from_value(v).ok()),
            makefile_category_filter: get_state(MakefileCategoryFilter::KEY)
                .and_then(|v| from_value(v).ok()),
            is_makefile_category_filter_active: get_state(IsMakefileCategoryFilterActive::KEY)
                .and_then(|v| from_value(v).ok()),
            pinned_makefile_tasks: get_state(PinnedMakefileTasks::KEY)
                .and_then(|v| from_value(v).ok()),
        }
    }
}

impl Default for VSCodeStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_async_trait]
impl ConfigurationManager for VSCodeStateManager {
    type UpdateError = StateManagerError;

    fn get<T: StateValue>(&self) -> Option<T> {
        get_state(T::KEY).and_then(|v| from_value(v).ok())
    }

    async fn update<T: StateValue>(&self, value: T) -> Result<(), Self::UpdateError> {
        // Get the old de-serialized value for comparison
        let update = if let Some(old_value_js) = get_state(T::KEY) {
            let old_value =
                from_value::<T>(old_value_js).map_err(StateManagerError::SerializationError)?;
            (old_value != value).then_some(value)
        } else {
            Some(value)
        };

        let value = if let Some(value) = update {
            to_value(&value).map_err(StateManagerError::SerializationError)?
        } else {
            JsValue::NULL
        };

        // Store in VS Code storage
        update_state(T::KEY.to_string(), value)
            .await
            .map_err(|e| StateManagerError::UpdateError(e.as_error_string()))?;

        self.call_subscriptions().await;

        Ok(())
    }

    fn subscribe(&mut self, on_change: impl Callback + 'static) {
        self.subscriptions.push(Box::new(on_change));
    }

    fn reset_subscriptions(&mut self) {
        self.subscriptions.clear();
    }
}
