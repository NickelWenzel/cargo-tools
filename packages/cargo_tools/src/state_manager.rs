use cargo_tools_macros::{wasm_async_trait, StateValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedPackage(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedBuildTarget(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedRunTarget(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedBenchmarkTarget(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedPlatformTarget(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedFeatures(Vec<String>);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct SelectedProfile(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct GroupByWorkspaceMember(bool);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct WorkspaceMemberFilter(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct TargetTypeFilter(Vec<String>);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct IsTargetTypeFilterActive(bool);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct ShowFeatures(bool);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct MakefileTaskFilter(String);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct MakefileCategoryFilter(Vec<String>);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct IsMakefileCategoryFilterActive(bool);

#[derive(Debug, StateValue, Serialize, Deserialize)]
pub struct PinnedMakefileTasks(Vec<String>);

pub trait StateValue: Serialize + for<'de> Deserialize<'de> + Send {
    type Value;
    const KEY: &'static str;
    fn into_value(self) -> Self::Value;
}

pub struct State {
    pub selected_package: SelectedPackage,
    pub selected_build_target: SelectedBuildTarget,
    pub selected_run_target: SelectedRunTarget,
    pub selected_benchmark_target: SelectedBenchmarkTarget,
    pub selected_platform_target: SelectedPlatformTarget,
    pub selected_features: SelectedFeatures,
    pub selected_profile: SelectedProfile,
    pub group_by_workspace_member: GroupByWorkspaceMember,
    pub workspace_member_filter: WorkspaceMemberFilter,
    pub target_type_filter: TargetTypeFilter,
    pub is_target_type_filter_active: IsTargetTypeFilterActive,
    pub show_features: ShowFeatures,
    pub makefile_task_filter: MakefileTaskFilter,
    pub makefile_category_filter: MakefileCategoryFilter,
    pub is_makefile_category_filter_active: IsMakefileCategoryFilterActive,
    pub pinned_makefile_tasks: PinnedMakefileTasks,
}

#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;

    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue>(&self, value: T) -> Result<(), Self::UpdateError>;

    fn subscribe(&self, on_change: impl AsyncFn(&State));
    fn reset_subscriptions();
}
