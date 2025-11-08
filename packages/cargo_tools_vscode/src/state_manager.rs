use async_trait::async_trait;

#[derive(Debug)]
pub struct SelectedPackage(String);

#[derive(Debug)]
pub struct SelectedBuildTarget(String);

#[derive(Debug)]
pub struct SelectedRunTarget(String);

#[derive(Debug)]
pub struct SelectedBenchmarkTarget(String);

#[derive(Debug)]
pub struct SelectedPlatformTarget(String);

#[derive(Debug)]
pub struct SelectedFeatures(Vec<String>);

#[derive(Debug)]
pub struct SelectedProfile(String);

#[derive(Debug)]
pub struct GroupByWorkspaceMember(bool);

#[derive(Debug)]
pub struct WorkspaceMemberFilter(String);

#[derive(Debug)]
pub struct TargetTypeFilter(Vec<String>);

#[derive(Debug)]
pub struct IsTargetTypeFilterActive(bool);

#[derive(Debug)]
pub struct ShowFeatures(bool);

#[derive(Debug)]
pub struct MakefileTaskFilter(String);

#[derive(Debug)]
pub struct MakefileCategoryFilter(Vec<String>);

#[derive(Debug)]
pub struct IsMakefileCategoryFilterActive(bool);

#[derive(Debug)]
pub struct PinnedMakefileTasks(Vec<String>);

pub trait StateValue {
    type Value;
    const KEY: &'static str;
    fn into_value() -> Self::Value;
}

#[async_trait]
pub trait StateManager {
    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update(&self, value: impl StateValue);
}
