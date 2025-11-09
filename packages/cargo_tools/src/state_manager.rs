use cargo_tools_macros::{wasm_async_trait, StateValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedPackage(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedBuildTarget(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedRunTarget(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedBenchmarkTarget(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedPlatformTarget(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedFeatures(Vec<String>);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct SelectedProfile(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct GroupByWorkspaceMember(bool);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct WorkspaceMemberFilter(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct TargetTypeFilter(Vec<String>);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct IsTargetTypeFilterActive(bool);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct ShowFeatures(bool);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct MakefileTaskFilter(String);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct MakefileCategoryFilter(Vec<String>);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct IsMakefileCategoryFilterActive(bool);

#[derive(Debug, Clone, PartialEq, StateValue, Serialize, Deserialize)]
pub struct PinnedMakefileTasks(Vec<String>);

pub trait StateValue: Serialize + for<'de> Deserialize<'de> + Send + PartialEq {
    type Value;
    const KEY: &'static str;
    fn into_value(self) -> Self::Value;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub selected_package: Option<SelectedPackage>,
    pub selected_build_target: Option<SelectedBuildTarget>,
    pub selected_run_target: Option<SelectedRunTarget>,
    pub selected_benchmark_target: Option<SelectedBenchmarkTarget>,
    pub selected_platform_target: Option<SelectedPlatformTarget>,
    pub selected_features: Option<SelectedFeatures>,
    pub selected_profile: Option<SelectedProfile>,
    pub group_by_workspace_member: Option<GroupByWorkspaceMember>,
    pub workspace_member_filter: Option<WorkspaceMemberFilter>,
    pub target_type_filter: Option<TargetTypeFilter>,
    pub is_target_type_filter_active: Option<IsTargetTypeFilterActive>,
    pub show_features: Option<ShowFeatures>,
    pub makefile_task_filter: Option<MakefileTaskFilter>,
    pub makefile_category_filter: Option<MakefileCategoryFilter>,
    pub is_makefile_category_filter_active: Option<IsMakefileCategoryFilterActive>,
    pub pinned_makefile_tasks: Option<PinnedMakefileTasks>,
}

#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;

    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue>(&self, value: T) -> Result<(), Self::UpdateError>;

    fn subscribe(&mut self, on_change: impl Callback);
    fn reset_subscriptions(&mut self);
}

#[wasm_async_trait]
pub trait Callback: 'static {
    async fn call(&self, state: &State);
}

#[wasm_async_trait]
impl<F: AsyncFn(&State) + 'static> Callback for F {
    async fn call(&self, state: &State) {
        (self)(state).await
    }
}
