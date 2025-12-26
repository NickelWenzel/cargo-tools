use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MakefileTaskFilter(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MakefileCategoryFilter(Vec<String>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsMakefileCategoryFilterActive(bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinnedMakefileTasks(Vec<String>);

pub struct VsCodeCargoMakeUi;

enum UiStateUpdate {
    MakefileTaskFilter(MakefileTaskFilter),
    MakefileCategoryFilter(MakefileCategoryFilter),
    IsMakefileCategoryFilterActive(IsMakefileCategoryFilterActive),
    PinnedMakefileTasks(PinnedMakefileTasks),
}

struct UiState {
    pub makefile_task_filter: Option<MakefileTaskFilter>,
    pub makefile_category_filter: Option<MakefileCategoryFilter>,
    pub is_makefile_category_filter_active: Option<IsMakefileCategoryFilterActive>,
    pub pinned_makefile_tasks: Option<PinnedMakefileTasks>,
}
