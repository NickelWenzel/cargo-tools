use std::{collections::HashMap, iter};

use cargo_metadata::{Metadata, Package, TargetKind};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::cargo::{
    Profile,
    command::{BuildSubTarget, RunSubTarget},
    metadata::TargetType,
};

/// A feature either targets the [Self::Workspace] or a named [Self::Package]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureTarget {
    Package(String),
    Workspace,
}

/// Mutually exclusive either [Self::All] or explicitly [Self::Some] features can be chosen
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

/// Holds an update for some aspect of [Config]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    SelectedPackage(Option<String>),
    SelectedBuildTarget(Option<BuildSubTarget>),
    SelectedRunTarget(Option<RunSubTarget>),
    SelectedBenchmarkTarget(Option<String>),
    SelectedPlatformTarget(Option<String>),
    SelectedFeatures {
        feature_target: FeatureTarget,
        features: Features,
    },
    SelectedProfile(Profile),
    Refresh(HashMap<String, PackageConfig>),
}

/// Represents the parameters which can serve as arguments for a `cargo` command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub selected_package: Option<String>,
    pub package_configs: HashMap<String, PackageConfig>,
    pub platform_target: Option<String>,
    pub profile: Profile,
    pub selected_features: Features,
}

impl Config {
    pub fn update(&mut self, update: Update) {
        match update {
            Update::SelectedPackage(package) => {
                self.selected_package = package;
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
            Update::SelectedFeatures {
                feature_target: feature_type,
                features,
            } => match feature_type {
                FeatureTarget::Package(package) => {
                    let s = self.package_configs.entry(package).or_default();
                    s.selected_features = features;
                }
                FeatureTarget::Workspace => self.selected_features = features,
            },
            Update::SelectedPlatformTarget(v) => self.platform_target = v,
            Update::SelectedProfile(v) => self.profile = v,
            Update::Refresh(package_selection) => {
                self.package_configs = package_selection
                // TODO: Should include selected workspace features
            }
        }
    }

    fn package_selection_mut(&mut self) -> Option<&mut PackageConfig> {
        let p = self.selected_package.clone()?;
        Some(self.package_configs.entry(p).or_default())
    }

    pub fn package_selection(&self) -> Option<&PackageConfig> {
        let p = self.selected_package.clone()?;
        self.package_configs.get(&p)
    }

    pub fn get<T>(&self, package: &str, get: impl Fn(&PackageConfig) -> Option<T>) -> Option<T> {
        self.package_configs.get(package).and_then(get)
    }

    pub fn args(&self, package: Option<&str>) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(platform) = self.platform_target.clone() {
            args.extend(["--target".to_string(), platform]);
        }
        args.extend(self.profile.cargo_args());

        let features = package
            .and_then(|p| self.package_configs.get(p))
            .map(|c| &c.selected_features)
            .unwrap_or(&self.selected_features);

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
            .map(|p| &p.selected_features)
            .unwrap_or(&self.selected_features)
            .clone()
    }

    fn selected_package(&self, metadata: &Metadata) -> Option<Package> {
        let selected = self.selected_package.as_ref()?;

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
        match self.selected_package(metadata) {
            Some(package) => features.chain(package.features.keys().cloned()).collect(),
            None => {
                let package_features = metadata
                    .workspace_packages()
                    .into_iter()
                    .flat_map(|package| package.features.keys().cloned())
                    .sorted()
                    .unique();
                features.chain(package_features).collect()
            }
        }
    }

    pub fn feature_target(&self) -> FeatureTarget {
        match &self.selected_package {
            Some(package) => FeatureTarget::Package(package.clone()),
            None => FeatureTarget::Workspace,
        }
    }
}

/// Represents the parameters which can serve as arguments for a `cargo` command which targets a specific package
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageConfig {
    pub build_target: Option<BuildSubTarget>,
    pub run_target: Option<RunSubTarget>,
    pub benchmark_target: Option<String>,
    pub selected_features: Features,
}

impl PackageConfig {
    pub fn build_target_matches(&self, target: TargetType, name: &str) -> bool {
        self.build_target
            .as_ref()
            .filter(|t| t.matches(target, name))
            .is_some()
    }

    pub fn run_target_matches(&self, target: TargetType, name: &str) -> bool {
        self.run_target
            .as_ref()
            .filter(|t| t.matches(target, name))
            .is_some()
    }

    pub fn bench_target_matches(&self, name: &str) -> bool {
        self.benchmark_target.as_deref() == Some(name)
    }
}
