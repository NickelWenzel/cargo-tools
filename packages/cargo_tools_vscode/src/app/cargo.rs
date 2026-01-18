pub mod command;

use std::iter;

use cargo_metadata::Metadata;
use cargo_tools::{
    app::cargo::{
        self,
        command::{BuildSubTarget, RunSubTarget},
        metadata::MetadataUpdate,
        selection::{self, Features},
    },
    profile::Profile,
};
use futures::{
    SinkExt, Stream, StreamExt,
    channel::mpsc::{Sender, channel},
};
use iced_headless::{Subscription, Task, stream};

use cargo::ui::Message as Msg;
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        Command, SelectInput,
        cargo::command::{CargoToolsCmd, register::register_cargo_commands},
    },
    runtime::CHANNEL_CAPACITY,
    vs_code_api::log,
};

#[derive(Debug, Clone)]
pub enum UiUpdate {
    CmdTx(Sender<CargoToolsCmd>),
    Cmd(CargoToolsCmd),
}

#[derive(Debug)]
pub struct Ui {
    data: CommandData,
    cmds: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineSettings {
    pub package_filter: PackageFilter,
    pub target_types_filter: TargetTypesFilter,
    pub grouping: Grouping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetTypesFilter {
    bin: bool,
    lib: bool,
    example: bool,
    benchmarks: bool,
    features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFilter(String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Grouping {
    Packages,
    TargetTypes,
}

#[derive(Debug, Clone)]
pub struct CommandData {
    pub metadata: UiMetadata,
    pub selection: selection::State,
}

impl CommandData {
    pub fn profiles(&self) -> SelectInput<Profile> {
        let options = self.metadata.profiles().to_vec();
        let current = vec![self.selection.profile.clone()];

        SelectInput { options, current }
    }

    pub fn packages(&self) -> SelectInput<Option<String>> {
        let options = self.metadata.package_options();
        let current = vec![self.selection.package.clone()];

        SelectInput { options, current }
    }

    pub fn build_target_options(&self) -> Option<SelectInput<Option<BuildSubTarget>>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let selection = &self.selection;

        let options = selection.build_target_options(metadata);
        let current = vec![
            selection
                .package_selection()
                .and_then(|s| s.build_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    pub fn run_target_options(&self) -> Option<SelectInput<Option<RunSubTarget>>> {
        let metadata = self.metadata.metadata.as_ref()?;

        let selection = &self.selection;

        let options = selection.run_target_options(metadata);
        let current = vec![
            selection
                .package_selection()
                .and_then(|s| s.run_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    pub fn bench_target_options(&self) -> Option<SelectInput<Option<String>>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let selection = &self.selection;

        let options = selection.bench_target_options(metadata);
        let current = vec![
            selection
                .package_selection()
                .and_then(|s| s.benchmark_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    pub fn feature_options(&self) -> Option<SelectInput<String>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let selection = &self.selection;

        let options = iter::once("All features".to_string())
            .chain(selection.feature_options(metadata))
            .collect::<Vec<_>>();
        let current = match selection.selected_features() {
            Features::All => ["All features".to_string()].to_vec(),
            Features::Some(features) => features,
        };

        Some(SelectInput { options, current })
    }
}

impl Ui {
    pub fn new(selection: selection::State) -> Self {
        let cmd_data = CommandData {
            metadata: UiMetadata::default(),
            selection,
        };

        Self {
            data: cmd_data,
            cmds: Vec::new(),
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
    type CustomUpdate = UiUpdate;

    fn update(&mut self, msg: Msg<Self::CustomUpdate>) -> Task<Msg<Self::CustomUpdate>> {
        log("Cargo Ui update received");
        match msg {
            Msg::Selection(update) => {
                self.data.selection.update(update);
                Task::none()
            }
            Msg::Metadata(update) => {
                match update {
                    MetadataUpdate::Metadata(metadata) => {
                        self.data.metadata.metadata = Some(metadata)
                    }
                    MetadataUpdate::Profiles(profiles) => self.data.metadata.profiles = profiles,
                    MetadataUpdate::NoCargoToml => self.data.metadata = UiMetadata::default(),
                    MetadataUpdate::FailedToRetrieve => {}
                };
                Task::none()
            }
            Msg::Task(_) => Task::none(),
            Msg::Custom(msg) => match msg {
                UiUpdate::CmdTx(tx) => {
                    self.cmds = register_cargo_commands(tx);
                    Task::none()
                }
                UiUpdate::Cmd(cmd) => self.process_cmd(cmd),
            },
        }
    }

    fn subscription(&self) -> Subscription<Msg<Self::CustomUpdate>> {
        Subscription::run(command_stream).map(Msg::Custom)
    }
}

fn command_stream() -> impl Stream<Item = UiUpdate> {
    stream::channel(CHANNEL_CAPACITY, async |mut out| {
        let (tx, mut rx) = channel(CHANNEL_CAPACITY);
        if let Err(e) = out.send(UiUpdate::CmdTx(tx.clone())).await {
            log(&format!(
                "Failed to send cargo ui command sender back to ui: {e:?}"
            ));
        }
        while let Some(msg) = rx.next().await {
            log(&format!("Sending command message to cargo Ui'{msg:?}'"));
            if let Err(e) = out.send(UiUpdate::Cmd(msg)).await {
                log(&format!(
                    "Failed to send command message to cargo UI '{e:?}'"
                ));
            }
        }
        log("Cargo Ui command stream closed unexpectedly.");
    })
}
