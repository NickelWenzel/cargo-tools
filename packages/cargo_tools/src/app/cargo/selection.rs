use std::collections::HashMap;

use cargo_metadata::Metadata;
use serde::{Deserialize, Serialize};

use crate::app::cargo::command::{BuildSubTarget, RunSubTarget};

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
pub enum Update {
    SelectedPackage(Option<String>),
    SelectedBuildTarget(Option<BuildSubTarget>),
    SelectedRunTarget(Option<RunSubTarget>),
    SelectedBenchmarkTarget(Option<String>),
    SelectedPlatformTarget(Option<String>),
    SelectedFeatures(Features),
    SelectedProfile(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct State {
    pub package: Option<String>,
    pub package_selection: HashMap<String, PackageSelection>,
    pub platform_target: Option<String>,
    pub profile: Option<String>,
}

impl State {
    pub fn update(&mut self, update: Update, metadata: &Metadata) {
        match update {
            Update::SelectedPackage(package) => {
                let Some(package) = package else {
                    return;
                };

                self.package = metadata
                    .workspace_packages()
                    .iter()
                    .find_map(|p| (p.name == package).then_some(p.id.to_string()));
            }
            Update::SelectedBuildTarget(v) => {
                if let Some(s) = self.package_selection() {
                    s.build_target = v;
                }
            }
            Update::SelectedRunTarget(v) => {
                if let Some(s) = self.package_selection() {
                    s.run_target = v;
                }
            }
            Update::SelectedBenchmarkTarget(v) => {
                if let Some(s) = self.package_selection() {
                    s.benchmark_target = v;
                }
            }
            Update::SelectedFeatures(v) => {
                if let Some(s) = self.package_selection() {
                    s.features = v;
                }
            }
            Update::SelectedPlatformTarget(v) => self.platform_target = v,
            Update::SelectedProfile(v) => self.profile = v,
        }
    }

    fn package_selection(&mut self) -> Option<&mut PackageSelection> {
        let p = self.package.clone()?;
        Some(self.package_selection.entry(p).or_default())
    }

    pub fn get<T>(&self, package: &str, get: impl Fn(&PackageSelection) -> Option<T>) -> Option<T> {
        if let Some(package) = self.package_selection.get(package) {
            get(package)
        } else {
            None
        }
    }

    pub fn append_platform_and_target(&self, mut args: Vec<String>) -> Vec<String> {
        if let Some(platform) = self.platform_target.clone() {
            args.push("--target".to_string());
            args.push(platform);
        }
        if let Some(profile) = self.profile.clone() {
            args.push("--profile".to_string());
            args.push(profile);
        }
        args
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageSelection {
    pub build_target: Option<BuildSubTarget>,
    pub run_target: Option<RunSubTarget>,
    pub benchmark_target: Option<String>,
    pub features: Features,
}
