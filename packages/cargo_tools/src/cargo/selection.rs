use std::{collections::HashMap, iter};

use cargo_metadata::{Metadata, Package, TargetKind};
use serde::{Deserialize, Serialize};

use crate::{
    cargo::{
        command::{BuildSubTarget, RunSubTarget},
        metadata::Target,
    },
    profile::Profile,
};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    SelectedPackage(Option<String>),
    SelectedBuildTarget(Option<BuildSubTarget>),
    SelectedRunTarget(Option<RunSubTarget>),
    SelectedBenchmarkTarget(Option<String>),
    SelectedPlatformTarget(Option<String>),
    SelectedFeatures(Features),
    SelectedProfile(Profile),
    Refresh(HashMap<String, PackageSelection>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct State {
    pub package: Option<String>,
    pub package_selection: HashMap<String, PackageSelection>,
    pub platform_target: Option<String>,
    pub profile: Profile,
    pub features: Features,
}

impl State {
    pub fn update(&mut self, update: Update) {
        match update {
            Update::SelectedPackage(package) => {
                self.package = package;
            }
            Update::SelectedBuildTarget(v) => {
                if let Some(s) = self.package_selection_mut() {
                    s.build_target = v;
                }
            }
            Update::SelectedRunTarget(v) => {
                if let Some(s) = self.package_selection_mut() {
                    s.run_target = v;
                }
            }
            Update::SelectedBenchmarkTarget(v) => {
                if let Some(s) = self.package_selection_mut() {
                    s.benchmark_target = v;
                }
            }
            Update::SelectedFeatures(v) => {
                if let Some(s) = self.package_selection_mut() {
                    s.features = v;
                } else {
                    self.features = v;
                }
            }
            Update::SelectedPlatformTarget(v) => self.platform_target = v,
            Update::SelectedProfile(v) => self.profile = v,
            Update::Refresh(package_selection) => self.package_selection = package_selection,
        }
    }

    fn package_selection_mut(&mut self) -> Option<&mut PackageSelection> {
        let p = self.package.clone()?;
        Some(self.package_selection.entry(p).or_default())
    }

    pub fn package_selection(&self) -> Option<&PackageSelection> {
        let p = self.package.clone()?;
        self.package_selection.get(&p)
    }

    pub fn get<T>(&self, package: &str, get: impl Fn(&PackageSelection) -> Option<T>) -> Option<T> {
        if let Some(package) = self.package_selection.get(package) {
            get(package)
        } else {
            None
        }
    }

    pub fn args(&self, for_package: bool) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(platform) = self.platform_target.clone() {
            args.extend(["--target".to_string(), platform]);
        }
        args.extend(self.profile.cargo_args());

        let features = if let Some(s) = self.package_selection()
            && for_package
        {
            &s.features
        } else {
            &self.features
        };

        match features {
            Features::All => args.push("--all-features".to_string()),
            Features::Some(items) if !items.is_empty() => {
                args.extend(["--features".to_string(), items.join(",")])
            }
            Features::Some(_) => {}
        };

        args
    }

    pub fn selected_features(&self) -> Features {
        self.package_selection()
            .map(|p| &p.features)
            .unwrap_or(&self.features)
            .clone()
    }

    fn selected_package(&self, metadata: &Metadata) -> Option<Package> {
        let selected = self.package.as_ref()?;

        metadata
            .workspace_packages()
            .into_iter()
            .find(|p| p.name == selected)
            .cloned()
    }

    pub fn build_target_options(&self, metadata: &Metadata) -> Vec<Option<BuildSubTarget>> {
        let Some(package) = self.selected_package(metadata) else {
            return Vec::new();
        };

        let targets = package.targets.iter().filter_map(|target| {
            let mut kind = target.kind.iter();
            if kind.clone().any(|k| matches!(k, TargetKind::Bin)) {
                return Some(BuildSubTarget::Bin(target.name.clone()));
            } else if kind.clone().any(|k| {
                matches!(
                    k,
                    TargetKind::Lib
                        | TargetKind::RLib
                        | TargetKind::DyLib
                        | TargetKind::CDyLib
                        | TargetKind::StaticLib
                        | TargetKind::ProcMacro
                )
            }) {
                return Some(BuildSubTarget::Lib(package.name.to_string()));
            } else if kind
                .clone()
                .any(|k: &TargetKind| matches!(k, TargetKind::Example))
            {
                return Some(BuildSubTarget::Example(target.name.clone()));
            } else if kind.any(|k| matches!(k, TargetKind::Bench)) {
                return Some(BuildSubTarget::Bench(target.name.clone()));
            }
            None
        });

        iter::once(None).chain(targets.map(Option::Some)).collect()
    }

    pub fn run_target_options(&self, metadata: &Metadata) -> Vec<Option<RunSubTarget>> {
        let Some(package) = self.selected_package(metadata) else {
            return Vec::new();
        };

        let targets = package.targets.iter().filter_map(|target| {
            let mut kind = target.kind.iter();
            if kind.clone().any(|k| matches!(k, TargetKind::Bin)) {
                return Some(RunSubTarget::Bin(target.name.clone()));
            } else if kind.any(|k| matches!(k, TargetKind::Example)) {
                return Some(RunSubTarget::Example(target.name.clone()));
            }
            None
        });

        iter::once(None).chain(targets.map(Option::Some)).collect()
    }

    pub fn bench_target_options(&self, metadata: &Metadata) -> Vec<Option<String>> {
        let Some(package) = self.selected_package(metadata) else {
            return Vec::new();
        };

        let targets = package.targets.iter().filter_map(|target| {
            if target.kind.iter().any(|k| matches!(k, TargetKind::Bench)) {
                return Some(target.name.clone());
            }
            None
        });

        iter::once(None).chain(targets.map(Option::Some)).collect()
    }

    pub fn feature_options(&self, metadata: &Metadata) -> Vec<String> {
        let features = iter::once("All features".to_string());
        if let Some(package) = self.selected_package(metadata) {
            features.chain(package.features.keys().cloned()).collect()
        } else {
            features.collect()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageSelection {
    pub build_target: Option<BuildSubTarget>,
    pub run_target: Option<RunSubTarget>,
    pub benchmark_target: Option<String>,
    pub features: Features,
}

impl PackageSelection {
    pub fn get(&self, target: Target) -> Option<String> {
        match target {
            Target::Lib => todo!(),
            Target::Bin => todo!(),
            Target::Example => todo!(),
            Target::Bench => todo!(),
        }
    }

    pub fn build_target_matches(&self, target: Target, name: &str) -> bool {
        self.build_target
            .as_ref()
            .filter(|t| t.matches(target, name))
            .is_some()
    }

    pub fn run_target_matches(&self, target: Target, name: &str) -> bool {
        self.run_target
            .as_ref()
            .filter(|t| t.matches(target, name))
            .is_some()
    }

    pub fn bench_target_matches(&self, name: &str) -> bool {
        self.benchmark_target.as_deref() == Some(name)
    }
}
