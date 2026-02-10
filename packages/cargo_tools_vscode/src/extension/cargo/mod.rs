pub mod command;

use std::iter;

use cargo_metadata::Metadata;
use cargo_tools::{
    cargo::{
        command::{BuildSubTarget, RunSubTarget},
        metadata::{MetadataUpdate, parse_metadata, parse_profiles, workspace_manifests},
        selection::{self, Features},
    },
    profile::Profile,
    runtime::Runtime as _,
};
use futures::channel::mpsc::{Sender, channel};
use iced_headless::Task;

use serde::{Deserialize, Serialize};

use crate::{
    extension::{
        base::{Base, send_file_changed},
        cargo::command::{Command, register_cargo_commands},
    },
    quick_pick::SelectInput,
    runtime::{CHANNEL_CAPACITY, VsCodeRuntime as Runtime},
    vs_code_api::{TsFileWatcher, set_cargo_context},
};

#[derive(Debug, Clone)]
pub enum Message {
    ManifestChanged,
    MetadataChanged(MetadataUpdate),
    SelectionChanged(selection::Update),
    SettingsChanged(SettingsUpdate),
    Cmd(Command),
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
    base: Base,
    cmd_tx: Sender<Command>,
}

impl Ui {
    /// Inits all data and update channels
    pub fn new(root_dir: String) -> (Self, Task<Message>) {
        // Init manifest file updates
        let (manifest_changed_tx, manifest_changed_rx) = channel(CHANNEL_CAPACITY);
        let file_watcher = TsFileWatcher::new(send_file_changed(manifest_changed_tx));

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let cmds = register_cargo_commands(cmd_tx.clone());

        let settings = Runtime::get_state(settings_key(&root_dir)).unwrap_or_default();
        let selection = Runtime::get_state(state_key(&root_dir)).unwrap_or_default();
        let data = CommandData {
            metadata: UiMetadata::default(),
            selection,
        };

        let base = Base {
            cmds,
            file_watcher,
            root_dir,
        };

        let this = Self {
            data,
            settings,
            base,
            cmd_tx,
        };

        // manifest update and cmd will run for the lifetime of the extension
        let manifest_update = Task::stream(manifest_changed_rx).map(|()| Message::ManifestChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        // metadata is to initially parse the Cargo.toml
        let metadata = this.parse_manifest();
        let tasks = Task::batch([manifest_update, cmd, metadata]);

        (this, tasks)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectionChanged(update) => {
                self.data.selection.update(update);
                Task::none()
            }
            Message::MetadataChanged(update) => match update {
                MetadataUpdate::Metadata(metadata) => {
                    // Update file watcher
                    let mut manifests = workspace_manifests(&metadata);
                    manifests.push(self.root_manifest());
                    self.base.file_watcher.watch_files(manifests);

                    self.data.metadata.metadata = Some(metadata);

                    Task::future(set_cargo_context(true)).discard()
                }
                MetadataUpdate::Profiles(profiles) => {
                    self.data.metadata.profiles = profiles;
                    Task::none()
                }
                MetadataUpdate::NoCargoToml => {
                    // Always check for mainfest in root dir
                    self.base
                        .file_watcher
                        .watch_files(vec![self.root_manifest()]);

                    self.data.metadata = UiMetadata::default();

                    Task::future(set_cargo_context(false)).discard()
                }
                // For invalid makefiles leave everything as is
                MetadataUpdate::FailedToRetrieve => Task::none(),
            },
            Message::ManifestChanged => self.parse_manifest(),
            Message::SettingsChanged(update) => self.update_settings(update),
            Message::Cmd(cmd) => self.process_cmd(cmd),
        }
    }

    fn update_settings(&mut self, update: SettingsUpdate) -> Task<Message> {
        let settings = &mut self.settings;
        match update {
            SettingsUpdate::PackageFilter(filter) => settings.package_filter = filter,
            SettingsUpdate::TargetTypesFilter(filter) => settings.target_types_filter = filter,
            SettingsUpdate::Grouping(grouping) => settings.grouping = grouping,
        }

        Task::future(Runtime::persist_state(
            settings_key(&self.base.root_dir),
            self.settings.clone(),
        ))
        .discard()
    }

    fn parse_manifest(&self) -> Task<Message> {
        let metadata = Task::future(parse_metadata::<Runtime>(self.root_manifest()));
        let profiles = Task::future(parse_profiles::<Runtime>(self.base.root_dir.clone()));
        Task::batch([metadata, profiles]).map(Message::MetadataChanged)
    }

    fn root_manifest(&self) -> String {
        format!("{}/Cargo.toml", self.base.root_dir)
    }
}

pub fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.cargo.ui_settings")
}

pub fn state_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.cargo.state")
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

impl PackageFilter {
    pub fn into_string(self) -> String {
        self.0
    }
}

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
struct CommandData {
    metadata: UiMetadata,
    selection: selection::State,
}

impl CommandData {
    fn profiles(&self) -> SelectInput<Profile> {
        let options = self.metadata.profiles().to_vec();
        let current = vec![self.selection.profile.clone()];

        SelectInput { options, current }
    }

    fn packages(&self) -> SelectInput<Option<String>> {
        let options = self.metadata.package_options();
        let current = vec![self.selection.package.clone()];

        SelectInput { options, current }
    }

    fn build_target_options(&self) -> Option<SelectInput<Option<BuildSubTarget>>> {
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

    fn run_target_options(&self) -> Option<SelectInput<Option<RunSubTarget>>> {
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

    fn bench_target_options(&self) -> Option<SelectInput<Option<String>>> {
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

    fn feature_options(&self) -> Option<SelectInput<String>> {
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
