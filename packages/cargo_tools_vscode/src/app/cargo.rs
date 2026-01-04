use std::{
    iter,
    sync::{Arc, Mutex},
};

use cargo_metadata::{Metadata, Package, TargetKind};
use cargo_tools::{
    app::cargo::{
        self,
        command::{BuildSubTarget, RunSubTarget},
        selection::{self},
    },
    profile::Profile,
};
use iced_headless::{Subscription, Task};

use cargo::ui::Message as Msg;

#[derive(Debug, Clone)]
pub struct Ui {
    pub data: Arc<Mutex<Data>>,
    pub selection: Arc<Mutex<selection::State>>,
}

impl Ui {
    pub fn new() -> Self {
        todo!()
    }

    fn selected_package(&self) -> Option<Package> {
        let Some(selected) = self.selection.lock().unwrap().package.clone() else {
            return None;
        };

        let guard = self.data.lock().unwrap();
        guard
            .metadata
            .workspace_packages()
            .into_iter()
            .find(|p| p.name == selected)
            .cloned()
    }

    pub fn build_target_options(&self) -> Vec<Option<BuildSubTarget>> {
        let Some(package) = self.selected_package() else {
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

    pub fn run_target_options(&self) -> Vec<Option<RunSubTarget>> {
        let Some(package) = self.selected_package() else {
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

    pub fn bench_target_options(&self) -> Vec<Option<String>> {
        let Some(package) = self.selected_package() else {
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

    pub fn feature_options(&self) -> Vec<String> {
        let Some(package) = self.selected_package() else {
            return Vec::new();
        };

        package.features.keys().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    metadata: Metadata,
    profiles: Vec<Profile>,
}

impl Data {
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn package_options(&self) -> Vec<Option<String>> {
        iter::once(None)
            .chain(
                self.metadata
                    .workspace_packages()
                    .iter()
                    .map(|p| Some(p.name.to_string())),
            )
            .collect()
    }
}

impl cargo::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, _update: Msg<Self>) -> Task<Msg<Self>> {
        todo!()
    }

    fn subscription(&self) -> Subscription<Msg<Self>> {
        Subscription::run(|| super::MSG_RX.lock().unwrap().clone()).filter_map(|msg| match msg {
            super::Message::Cargo(msg) => Some(msg),
            super::Message::CargoMake(_) => None,
        })
    }
}
