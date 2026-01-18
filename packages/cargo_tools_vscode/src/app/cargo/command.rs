use cargo_tools::app::cargo::{
    command::{BuildTarget, RunTarget},
    selection::Update,
};
use serde::de::DeserializeOwned;
use wasm_bindgen_futures::js_sys::Array;

use crate::{app::cargo::TargetTypesFilterUpdate, vs_code_api::log};

pub mod process;
pub mod register;

#[derive(Debug, Clone)]
pub enum CargoToolsCmd {
    SelectProfile,
    SelectPackage,
    SelectBuildTarget,
    SelectRunTarget,
    SelectBenchmarkTarget,
    SelectPlatformTarget,
    SelectFeatures,
    InstallPlatformTarget,
    SetRustAnalyzerCheckTargets,
    BuildDocs,
    Refresh,
    Clean,
    Build,
    Run,
    Debug,
    Test,
    Bench,
    ProjectOutline(ProjectOutline),
}

pub type CargoCmdFn = fn(Array) -> Option<CargoToolsCmd>;

impl CargoToolsCmd {
    pub const fn all() -> [(&'static str, CargoCmdFn); 31] {
        use ProjectOutline as PO;
        [
            ("cargo-tools.selectProfile", |_| Some(Self::SelectProfile)),
            ("cargo-tools.selectPackage", |_| Some(Self::SelectPackage)),
            ("cargo-tools.selectBuildTarget", |_| {
                Some(Self::SelectBuildTarget)
            }),
            ("cargo-tools.selectRunTarget", |_| {
                Some(Self::SelectRunTarget)
            }),
            ("cargo-tools.selectBenchmarkTarget", |_| {
                Some(Self::SelectBenchmarkTarget)
            }),
            ("cargo-tools.selectPlatformTarget", |_| {
                Some(Self::SelectPlatformTarget)
            }),
            ("cargo-tools.installPlatformTarget", |_| {
                Some(Self::InstallPlatformTarget)
            }),
            ("cargo-tools.setRustAnalyzerCheckTargets", |_| {
                Some(Self::SetRustAnalyzerCheckTargets)
            }),
            ("cargo-tools.buildDocs", |_| Some(Self::BuildDocs)),
            ("cargo-tools.selectFeatures", |_| Some(Self::SelectFeatures)),
            ("cargo-tools.refresh", |_| Some(Self::Refresh)),
            ("cargo-tools.clean", |_| Some(Self::Clean)),
            ("cargo-tools.projectStatus.build", |_| Some(Self::Build)),
            ("cargo-tools.projectStatus.run", |_| Some(Self::Run)),
            ("cargo-tools.projectStatus.debug", |_| Some(Self::Debug)),
            ("cargo-tools.projectStatus.test", |_| Some(Self::Test)),
            ("cargo-tools.projectStatus.bench", |_| Some(Self::Bench)),
            ("cargo-tools.projectOutline.select", |arg| {
                PO::from_update(PO::Select, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.unselect", |arg| {
                PO::from_update(PO::Unselect, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.build", |arg| {
                PO::from_build_target(PO::Build, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.test", |arg| {
                PO::from_optional_str(PO::Test, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.clean", |arg| {
                PO::from_optional_str(PO::Clean, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.run", |arg| {
                PO::from_run_target(PO::Run, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.debug", |arg| {
                PO::from_run_target(PO::Debug, arg).map(PO::to_cmd)
            }),
            ("cargo-tools.projectOutline.bench", |arg| {
                PO::from_optional_str(PO::Bench, arg).map(PO::to_cmd)
            }),
            (
                "cargo-tools.projectOutline.setWorkspaceMemberFilter",
                |_| Some(PO::SelectWorkspaceMemberFilter.to_cmd()),
            ),
            (
                "cargo-tools.projectOutline.editWorkspaceMemberFilter",
                |arg| PO::from_str(PO::EditWorkspaceMemberFilter, arg).map(PO::to_cmd),
            ),
            ("cargo-tools.projectOutline.showTargetTypeFilter", |_| {
                Some(PO::SelectTargetTypeFilter.to_cmd())
            }),
            (
                "cargo-tools.projectOutline.editWorkspaceMemberFilter",
                |arg| {
                    PO::from_target_types_filter_update(PO::EditTargetTypeFilter, arg)
                        .map(PO::to_cmd)
                },
            ),
            ("cargo-tools.projectOutline.clearAllFilters", |_| {
                Some(PO::ClearAllFilters.to_cmd())
            }),
            (
                "cargo-tools.projectOutline.toggleWorkspaceMemberGrouping",
                |_| Some(PO::ToggleWorkspaceMemberGrouping.to_cmd()),
            ),
        ]
    }
}

#[derive(Debug, Clone)]
pub enum ProjectOutline {
    Select(Update),
    Unselect(Update),
    Build(Option<BuildTarget>),
    Test(Option<String>),
    Clean(Option<String>),
    Run(RunTarget),
    Debug(RunTarget),
    Bench(Option<String>),
    SelectWorkspaceMemberFilter,
    EditWorkspaceMemberFilter(String),
    SelectTargetTypeFilter,
    EditTargetTypeFilter(TargetTypesFilterUpdate),
    ClearAllFilters,
    ToggleWorkspaceMemberGrouping,
}

impl ProjectOutline {
    pub const fn to_cmd(self) -> CargoToolsCmd {
        CargoToolsCmd::ProjectOutline(self)
    }

    pub fn from_update(cmd: fn(Update) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_build_target(cmd: fn(Option<BuildTarget>) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_run_target(cmd: fn(RunTarget) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_optional_str(cmd: fn(Option<String>) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_str(cmd: fn(String) -> Self, arg: Array) -> Option<Self> {
        take_first(arg).map(cmd)
    }

    pub fn from_target_types_filter_update(
        cmd: fn(TargetTypesFilterUpdate) -> Self,
        arg: Array,
    ) -> Option<Self> {
        take_first(arg).map(cmd)
    }
}

fn take_first<T: DeserializeOwned>(array: Array) -> Option<T> {
    match serde_wasm_bindgen::from_value(array.get(0)) {
        Ok(v) => Some(v),
        Err(e) => {
            log(&format!("Failed to deserialize update: {e}"));
            None
        }
    }
}
