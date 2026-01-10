use std::{
    iter,
    sync::{Arc, Mutex},
};

use async_broadcast::{Receiver, broadcast};
use cargo_metadata::Metadata;
use cargo_tools::{
    app::cargo::{
        self,
        command::{BuildSubTarget, RunSubTarget},
        selection::{self, Features},
    },
    profile::Profile,
};
use iced_headless::{Subscription, Task};

use cargo::ui::Message as Msg;

use crate::{
    app::StaticHashStream,
    command::{Command, SelectInput, register_cargo_commands},
    vs_code_api::log,
};

#[derive(Debug)]
pub struct Ui {
    cmd_data: CommandData,
    rx: Receiver<Msg<()>>,
    _cmds: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct CommandData {
    pub metadata: Arc<Mutex<UiMetadata>>,
    pub selection: Arc<Mutex<selection::State>>,
}

impl CommandData {
    pub fn profiles(&self) -> SelectInput<Profile> {
        let options = self.metadata.lock().unwrap().profiles().to_vec();
        let current = vec![self.selection.lock().unwrap().profile.clone()];

        SelectInput { options, current }
    }

    pub fn packages(&self) -> SelectInput<Option<String>> {
        let options = self.metadata.lock().unwrap().package_options();
        let current = vec![self.selection.lock().unwrap().package.clone()];

        SelectInput { options, current }
    }

    pub fn build_target_options(&self) -> Option<SelectInput<Option<BuildSubTarget>>> {
        let ui_metadata_guard = self.metadata.lock().unwrap();
        let metadata = ui_metadata_guard.metadata.as_ref()?;

        let selection_guard = self.selection.lock().unwrap();

        let options = selection_guard.build_target_options(metadata);
        let current = vec![
            selection_guard
                .package_selection()
                .and_then(|s| s.build_target.clone()),
        ];

        let input = SelectInput { options, current };
        log(&format!(
            "Build options '{input:?}' for current selection '{selection_guard:?}'"
        ));

        Some(input)
    }

    pub fn run_target_options(&self) -> Option<SelectInput<Option<RunSubTarget>>> {
        let ui_metadata_guard = self.metadata.lock().unwrap();
        let metadata = ui_metadata_guard.metadata.as_ref()?;

        let selection_guard = self.selection.lock().unwrap();

        let options = selection_guard.run_target_options(metadata);
        let current = vec![
            selection_guard
                .package_selection()
                .and_then(|s| s.run_target.clone()),
        ];

        let input = SelectInput { options, current };
        log(&format!(
            "Run options '{input:?}' for current selection '{selection_guard:?}'"
        ));

        Some(input)
    }

    pub fn bench_target_options(&self) -> Option<SelectInput<Option<String>>> {
        let ui_metadata_guard = self.metadata.lock().unwrap();
        let metadata = ui_metadata_guard.metadata.as_ref()?;

        let selection_guard = self.selection.lock().unwrap();

        let options = selection_guard.bench_target_options(metadata);
        let current = vec![
            selection_guard
                .package_selection()
                .and_then(|s| s.benchmark_target.clone()),
        ];

        let input = SelectInput { options, current };
        log(&format!(
            "Benchmark options '{input:?}' for current selection '{selection_guard:?}'"
        ));

        Some(input)
    }

    pub fn feature_options(&self) -> Option<SelectInput<String>> {
        let ui_metadata_guard = self.metadata.lock().unwrap();
        let metadata = ui_metadata_guard.metadata.as_ref()?;

        let selection_guard = self.selection.lock().unwrap();
        let options = iter::once("All features".to_string())
            .chain(selection_guard.feature_options(metadata))
            .collect::<Vec<_>>();
        let current = match selection_guard.selected_features() {
            Features::All => ["All features".to_string()].to_vec(),
            Features::Some(features) => features,
        };

        let input = SelectInput { options, current };
        log(&format!(
            "Feature options '{input:?}' for current selection '{selection_guard:?}'"
        ));

        Some(input)
    }
}

impl Ui {
    pub fn new(selection: selection::State) -> Self {
        let (tx, rx) = broadcast(100);
        let data = CommandData {
            metadata: Arc::new(Mutex::new(UiMetadata::default())),
            selection: Arc::new(Mutex::new(selection)),
        };

        Self {
            cmd_data: data.clone(),
            rx,
            _cmds: register_cargo_commands(tx, data),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiMetadata {
    pub metadata: Option<Metadata>,
    pub profiles: Vec<Profile>,
}

impl UiMetadata {
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn package_options(&self) -> Vec<Option<String>> {
        let Some(metadata) = &self.metadata else {
            return Vec::new();
        };
        iter::once(None)
            .chain(
                metadata
                    .workspace_packages()
                    .iter()
                    .map(|p| Some(p.name.to_string())),
            )
            .collect()
    }
}

impl cargo::ui::Ui for Ui {
    type CustomUpdate = ();

    fn update(&mut self, msg: Msg<Self::CustomUpdate>) -> Task<Msg<Self::CustomUpdate>> {
        log("Cargo Ui update received");
        match msg {
            Msg::Selection(update) => {
                log(&format!(
                    "Cargo Ui update received: new selection '{update:?}'"
                ));
                self.cmd_data.selection.lock().unwrap().update(update)
            }
            Msg::Metadata(update) => match update {
                cargo::metadata::MetadataUpdate::Metadata(metadata) => {
                    log("Cargo Ui update received: new metadata");
                    self.cmd_data.metadata.lock().unwrap().metadata = Some(metadata)
                }
                cargo::metadata::MetadataUpdate::Profiles(profiles) => {
                    log("Cargo Ui update received: new profiles");
                    self.cmd_data.metadata.lock().unwrap().profiles = profiles
                }
                cargo::metadata::MetadataUpdate::NoCargoToml => {
                    *self.cmd_data.metadata.lock().unwrap() = UiMetadata::default()
                }
                cargo::metadata::MetadataUpdate::FailedToRetrieve => {}
            },
            Msg::Task(_) => {}
            Msg::Custom(_) => {}
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>> {
        let stream = StaticHashStream::new(self.rx.clone(), "vscode_cargo");
        Subscription::run_with(stream, |stream| stream.clone())
    }
}
