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
    runtime::Runtime as _,
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
        CargoMsg, Command, SelectInput,
        cargo::command::{CargoToolsCmd, register::register_cargo_commands},
    },
    runtime::{CHANNEL_CAPACITY, VsCodeRuntime as Runtime},
    vs_code_api::log,
};

#[derive(Debug, Clone)]
pub enum UiMessage {
    CmdTx(Sender<CargoToolsCmd>),
    Cmd(CargoToolsCmd),
    Settings(SettingsUpdate),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    PackageFilter(PackageFilter),
    TargetTypesFilter(TargetTypesFilter),
    Grouping(Grouping),
}

#[derive(Debug)]
pub struct Ui {
    data: CommandData,
    settings: OutlineSettings,
    cmds: Vec<Command>,
    root_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutlineSettings {
    pub package_filter: PackageFilter,
    pub target_types_filter: TargetTypesFilter,
    pub grouping: Grouping,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TargetTypesFilter {
    bin: bool,
    lib: bool,
    example: bool,
    benchmarks: bool,
    features: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TargetTypesFilterUpdate {
    Bin(bool),
    Lib(bool),
    Example(bool),
    Benchmarks(bool),
    Features(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageFilter(String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Grouping {
    #[default]
    Packages,
    TargetTypes,
}

impl Grouping {
    pub fn toggle(self) -> Self {
        match self {
            Grouping::Packages => Grouping::TargetTypes,
            Grouping::TargetTypes => Grouping::Packages,
        }
    }
}

#[derive(Debug, Clone, Default)]
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
    pub fn new(root_dir: String) -> Self {
        Self {
            data: CommandData::default(),
            cmds: Vec::new(),
            settings: OutlineSettings::default(),
            root_dir,
        }
    }

    fn update_settings(&mut self, update: SettingsUpdate) -> Task<CargoMsg> {
        let settings = &mut self.settings;
        match update {
            SettingsUpdate::PackageFilter(filter) => settings.package_filter = filter,
            SettingsUpdate::TargetTypesFilter(filter) => settings.target_types_filter = filter,
            SettingsUpdate::Grouping(grouping) => settings.grouping = grouping,
        }

        Task::future(Runtime::persist_state(
            self.settings_key(),
            self.settings.clone(),
        ))
        .discard()
    }

    pub fn settings_key(&self) -> String {
        format!("{}.cargo_tools.cargo.ui_settings", self.root_dir)
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
    type CustomUpdate = UiMessage;

    fn update(&mut self, msg: CargoMsg) -> Task<CargoMsg> {
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
                UiMessage::CmdTx(tx) => {
                    self.cmds = register_cargo_commands(tx);
                    Task::none()
                }
                UiMessage::Cmd(cmd) => self.process_cmd(cmd),
                UiMessage::Settings(update) => self.update_settings(update),
            },
            Msg::RootDirUpdate(root_dir) => {
                self.root_dir = root_dir;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<CargoMsg> {
        Subscription::run(command_stream).map(Msg::Custom)
    }
}

fn command_stream() -> impl Stream<Item = UiMessage> {
    stream::channel(CHANNEL_CAPACITY, async |mut out| {
        let (tx, mut rx) = channel(CHANNEL_CAPACITY);
        if let Err(e) = out.send(UiMessage::CmdTx(tx.clone())).await {
            log(&format!(
                "Failed to send cargo ui command sender back to ui: {e:?}"
            ));
        }
        while let Some(msg) = rx.next().await {
            log(&format!("Sending command message to cargo Ui'{msg:?}'"));
            if let Err(e) = out.send(UiMessage::Cmd(msg)).await {
                log(&format!(
                    "Failed to send command message to cargo UI '{e:?}'"
                ));
            }
        }
        log("Cargo Ui command stream closed unexpectedly.");
    })
}
