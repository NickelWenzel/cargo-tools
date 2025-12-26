use std::collections::HashMap;

use cargo_metadata::{Metadata, PackageId};
use serde::{Deserialize, Serialize};

use crate::app::command::{BuildSubTarget, RunSubTarget};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Features {
    All,
    Some(Vec<String>),
}

impl Default for Features {
    fn default() -> Self {
        Self::Some(Vec::default())
    }
}

#[derive(Debug, Clone)]
pub enum SelectionUpdate {
    SelectedPackage(Option<String>),
    SelectedBuildTarget(Option<BuildSubTarget>),
    SelectedRunTarget(Option<RunSubTarget>),
    SelectedBenchmarkTarget(Option<String>),
    SelectedPlatformTarget(Option<String>),
    SelectedFeatures(Features),
    SelectedProfile(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Selection {
    pub package: Option<PackageId>,
    pub package_selection: HashMap<PackageId, PackageSelection>,
    pub platform_target: Option<String>,
    pub profile: Option<String>,
}

impl Selection {
    pub fn update(&mut self, update: SelectionUpdate, metadata: &Metadata) {
        match update {
            SelectionUpdate::SelectedPackage(package) => {
                let Some(package) = package else {
                    return;
                };

                self.package = metadata
                    .workspace_packages()
                    .iter()
                    .find_map(|p| (p.name == package).then_some(p.id.clone()));
            }
            SelectionUpdate::SelectedBuildTarget(v) => {
                if let Some(s) = self.package_selection() {
                    s.build_target = v;
                }
            }
            SelectionUpdate::SelectedRunTarget(v) => {
                if let Some(s) = self.package_selection() {
                    s.run_target = v;
                }
            }
            SelectionUpdate::SelectedBenchmarkTarget(v) => {
                if let Some(s) = self.package_selection() {
                    s.benchmark_target = v;
                }
            }
            SelectionUpdate::SelectedFeatures(v) => {
                if let Some(s) = self.package_selection() {
                    s.features = v;
                }
            }
            SelectionUpdate::SelectedPlatformTarget(v) => self.platform_target = v,
            SelectionUpdate::SelectedProfile(v) => self.profile = v,
        }
    }

    fn package_selection(&mut self) -> Option<&mut PackageSelection> {
        let p = self.package.clone()?;
        Some(self.package_selection.entry(p).or_default())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageSelection {
    pub build_target: Option<BuildSubTarget>,
    pub run_target: Option<RunSubTarget>,
    pub benchmark_target: Option<String>,
    pub features: Features,
}
