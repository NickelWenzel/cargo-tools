use cargo_tools_macros::wasm_async_trait;
use serde::{Deserialize, Serialize};

use crate::{configuration_handler::ConfigurationManager, runtime::Runtime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedPackage(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedBuildTarget(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedRunTarget(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedBenchmarkTarget(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedPlatformTarget(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedFeatures(Vec<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedProfile(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupByWorkspaceMember(bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMemberFilter(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetTypeFilter(Vec<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsTargetTypeFilterActive(bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShowFeatures(bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MakefileTaskFilter(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MakefileCategoryFilter(Vec<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsMakefileCategoryFilterActive(bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinnedMakefileTasks(Vec<String>);

#[derive(Debug, Clone)]
pub enum StateUpdate {
    SelectedPackage(SelectedPackage),
    SelectedBuildTarget(SelectedBuildTarget),
    SelectedRunTarget(SelectedRunTarget),
    SelectedBenchmarkTarget(SelectedBenchmarkTarget),
    SelectedPlatformTarget(SelectedPlatformTarget),
    SelectedFeatures(SelectedFeatures),
    SelectedProfile(SelectedProfile),
    GroupByWorkspaceMember(GroupByWorkspaceMember),
    WorkspaceMemberFilter(WorkspaceMemberFilter),
    TargetTypeFilter(TargetTypeFilter),
    IsTargetTypeFilterActive(IsTargetTypeFilterActive),
    ShowFeatures(ShowFeatures),
    MakefileTaskFilter(MakefileTaskFilter),
    MakefileCategoryFilter(MakefileCategoryFilter),
    IsMakefileCategoryFilterActive(IsMakefileCategoryFilterActive),
    PinnedMakefileTasks(PinnedMakefileTasks),
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
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

pub struct StateHandler;

#[wasm_async_trait]
impl ConfigurationManager for StateHandler {
    type Configuration = State;
    type ConfigurationUpdate = StateUpdate;

    async fn update_root_dir<RuntimeT: Runtime>(root_dir: String) -> State {
        RuntimeT::update_state_context(root_dir).await
    }
    async fn apply_update<RuntimeT: Runtime>(update: StateUpdate) -> State {
        RuntimeT::update_state(update).await
    }
}
