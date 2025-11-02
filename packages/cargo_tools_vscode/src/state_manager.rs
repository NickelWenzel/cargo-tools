use wasm_bindgen::prelude::*;

use crate::vs_code_api::StateManagerTS;

pub enum StateValue {
    SelectedPackage(String),
    SelectedBuildTarget(String),
    SelectedRunTarget(String),
    SelectedBenchmarkTarget(String),
    SelectedPlatformTarget(String),
    SelectedFeatures(Vec<String>),
    SelectedProfile(String),
    GroupByWorkspaceMember(bool),
    WorkspaceMemberFilter(String),
    TargetTypeFilter(Vec<String>),
    IsTargetTypeFilterActive(bool),
    ShowFeatures(bool),
    MakefileTaskFilter(String),
    MakefileCategoryFilter(Vec<String>),
    IsMakefileCategoryFilterActive(bool),
    PinnedMakefileTasks(Vec<String>),
}

#[wasm_bindgen]
pub struct StateManager(StateManagerTS);

#[wasm_bindgen]
impl StateManager {
    #[wasm_bindgen(constructor)]
    pub fn new(state_manager: StateManagerTS) -> Self {
        Self(state_manager)
    }

    pub fn get_selected_package(&self) -> Option<String> {
        self.0.get("selectedPackage").and_then(|v| v.as_string())
    }

    pub async fn set_selected_package(&self, selected_package: &str) {
        self.0
            .update("selectedPackage", &JsValue::from(selected_package))
            .await;
    }

    pub fn get_selected_build_target(&self) -> Option<String> {
        self.0
            .get("selectedBuildTarget")
            .and_then(|v| v.as_string())
    }

    pub async fn set_selected_build_target(&self, selected_build_target: &str) {
        self.0
            .update("selectedBuildTarget", &JsValue::from(selected_build_target))
            .await;
    }

    pub fn get_selected_run_target(&self) -> Option<String> {
        self.0.get("selectedRunTarget").and_then(|v| v.as_string())
    }

    pub async fn set_selected_run_target(&self, selected_run_target: &str) {
        self.0
            .update("selectedRunTarget", &JsValue::from(selected_run_target))
            .await;
    }

    pub fn get_selected_benchmark_target(&self) -> Option<String> {
        self.0
            .get("selectedBenchmarkTarget")
            .and_then(|v| v.as_string())
    }

    pub async fn set_selected_benchmark_target(&self, selected_benchmark_target: &str) {
        self.0
            .update(
                "selectedBenchmarkTarget",
                &JsValue::from(selected_benchmark_target),
            )
            .await;
    }

    pub fn get_selected_platform_target(&self) -> Option<String> {
        self.0
            .get("selectedPlatformTarget")
            .and_then(|v| v.as_string())
    }

    pub async fn set_selected_platform_target(&self, selected_platform_target: &str) {
        self.0
            .update(
                "selectedPlatformTarget",
                &JsValue::from(selected_platform_target),
            )
            .await;
    }

    pub fn get_selected_features(&self) -> Option<Vec<String>> {
        self.0
            .get("selectedFeatures")
            .and_then(|v| serde_wasm_bindgen::from_value(v).ok())
    }

    pub async fn set_selected_features(&self, selected_features: Vec<String>) {
        self.0
            .update("selectedFeatures", &JsValue::from(selected_features))
            .await;
    }

    pub fn get_selected_profile(&self) -> Option<String> {
        self.0.get("selectedProfile").and_then(|v| v.as_string())
    }

    pub async fn set_selected_profile(&self, selected_profile: &str) {
        self.0
            .update("selectedProfile", &JsValue::from(selected_profile))
            .await;
    }

    pub fn get_group_by_workspace_member(&self) -> Option<bool> {
        self.0
            .get("groupByWorkspaceMember")
            .and_then(|v| v.as_bool())
    }

    pub async fn set_group_by_workspace_member(&self, group_by: bool) {
        self.0
            .update("groupByWorkspaceMember", &JsValue::from(group_by))
            .await;
    }

    pub fn get_workspace_member_filter(&self) -> Option<String> {
        self.0
            .get("workspaceMemberFilter")
            .and_then(|v| v.as_string())
    }

    pub async fn set_workspace_member_filter(&self, workspace_member_filter: &str) {
        self.0
            .update(
                "workspaceMemberFilter",
                &JsValue::from(workspace_member_filter),
            )
            .await;
    }

    pub fn get_target_type_filter(&self) -> Option<Vec<String>> {
        self.0
            .get("targetTypeFilter")
            .and_then(|v| serde_wasm_bindgen::from_value(v).ok())
    }

    pub async fn set_target_type_filter(&self, target_type_filters: Vec<String>) {
        self.0
            .update("targetTypeFilter", &JsValue::from(target_type_filters))
            .await;
    }

    pub fn get_is_target_type_filter_active(&self) -> Option<bool> {
        self.0
            .get("isTargetTypeFilterActive")
            .and_then(|v| v.as_bool())
    }

    pub async fn set_is_target_type_filter_active(&self, is_active: bool) {
        self.0
            .update("isTargetTypeFilterActive", &JsValue::from(is_active))
            .await;
    }

    pub fn get_show_features(&self) -> Option<bool> {
        self.0.get("showFeatures").and_then(|v| v.as_bool())
    }

    pub async fn set_show_features(&self, show_features: bool) {
        self.0
            .update("showFeatures", &JsValue::from(show_features))
            .await;
    }

    pub fn get_makefile_task_filter(&self) -> Option<String> {
        self.0.get("makefileTaskFilter").and_then(|v| v.as_string())
    }

    pub async fn set_makefile_task_filter(&self, make_task_filter: &str) {
        self.0
            .update("makefileTaskFilter", &JsValue::from(make_task_filter))
            .await;
    }

    pub fn get_makefile_category_filter(&self) -> Option<Vec<String>> {
        self.0
            .get("makefileCategoryFilter")
            .and_then(|v| serde_wasm_bindgen::from_value(v).ok())
    }

    pub async fn set_makefile_category_filter(&self, makefile_category_filter: Vec<String>) {
        self.0
            .update(
                "makefileCategoryFilter",
                &JsValue::from(makefile_category_filter),
            )
            .await;
    }

    pub fn get_is_makefile_category_filter_active(&self) -> Option<bool> {
        self.0
            .get("isMakefileCategoryFilterActive")
            .map(|v| v.as_bool().unwrap_or(false))
    }

    pub async fn set_is_makefile_category_filter_active(&self, is_active: bool) {
        self.0
            .update("isMakefileCategoryFilterActive", &JsValue::from(is_active))
            .await;
    }

    pub fn get_pinned_makefile_tasks(&self) -> Option<Vec<String>> {
        self.0
            .get("pinnedMakefileTasks")
            .and_then(|v| serde_wasm_bindgen::from_value(v).ok())
    }

    pub async fn set_pinned_makefile_tasks(&self, tasks: Vec<String>) {
        self.0
            .update("pinnedMakefileTasks", &JsValue::from(tasks))
            .await;
    }
}
