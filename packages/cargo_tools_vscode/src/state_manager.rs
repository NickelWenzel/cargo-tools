use cargo_tools_vscode_macros::{wasm_async_trait, StateValue};
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

pub trait StateValue: Serialize + for<'de> Deserialize<'de> {
    type Value;
    const KEY: &'static str;
    fn into_value(self) -> Self::Value;
}

#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;

    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue + 'static>(&self, value: T) -> Result<(), Self::UpdateError>;
}
