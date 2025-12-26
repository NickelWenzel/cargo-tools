use serde::{Deserialize, Serialize};

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

pub struct VsCodeCargoSettingsUi;

struct UiSettings {
    pub group_by_workspace_member: Option<GroupByWorkspaceMember>,
    pub workspace_member_filter: Option<WorkspaceMemberFilter>,
    pub target_type_filter: Option<TargetTypeFilter>,
    pub is_target_type_filter_active: Option<IsTargetTypeFilterActive>,
    pub show_features: Option<ShowFeatures>,
}

enum UiSettingsUpdate {
    GroupByWorkspaceMember(GroupByWorkspaceMember),
    WorkspaceMemberFilter(WorkspaceMemberFilter),
    TargetTypeFilter(TargetTypeFilter),
    IsTargetTypeFilterActive(IsTargetTypeFilterActive),
    ShowFeatures(ShowFeatures),
}
