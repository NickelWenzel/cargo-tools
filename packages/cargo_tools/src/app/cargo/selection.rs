use std::collections::HashMap;

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
        }
    }

    fn package_selection_mut(&mut self) -> Option<&mut PackageSelection> {
        let p = self.package.clone()?;
        Some(self.package_selection.entry(p).or_default())
    }

    fn package_selection(&self) -> Option<&PackageSelection> {
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
        if let Some(profile) = self.profile.clone() {
            args.extend(["--profile".to_string(), profile]);
        }

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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageSelection {
    pub build_target: Option<BuildSubTarget>,
    pub run_target: Option<RunSubTarget>,
    pub benchmark_target: Option<String>,
    pub features: Features,
}
