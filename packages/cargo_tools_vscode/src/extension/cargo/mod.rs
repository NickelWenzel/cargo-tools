pub mod command;
pub mod ui;

use std::{iter, path::Path};

use cargo_metadata::Metadata;
use cargo_tools::cargo::{
    Config, ConfigUpdate, Profile,
    command::{BuildSubTarget, RunSubTarget},
    config::Features,
    metadata::{
        CondensedPackage, CondensedTarget, MetadataUpdate, Target, parse_metadata, parse_profiles,
        workspace_manifests,
    },
};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_viewless::Task;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    extension::{
        base::{Base, send_file_changed},
        cargo::{
            command::{Command, register_cargo_commands},
            ui::{
                CargoConfigurationTreeProviderHandler, CargoOutlineTreeProviderHandler,
                ConfigUiRequest, NodeData, OutlineNodeData, OutlineUiRequest,
            },
        },
    },
    quick_pick::SelectInput,
    runtime::{
        CHANNEL_CAPACITY, exec_vs_code, get_state_vs_code, persist_state_vs_code, read_file_vs_code,
    },
    vs_code_api::{
        CargoConfigurationTreeProvider, CargoOutlineTreeProvider, TsFileWatcher, set_cargo_context,
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    ManifestChanged,
    MetadataChanged(MetadataUpdate),
    SelectionChanged(ConfigUpdate),
    SettingsChanged(SettingsUpdate),
    Cmd(Command),
    ConfigUiRequest(ConfigUiRequest),
    OutlineUiRequest(OutlineUiRequest),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    PackageFilter(String),
    TargetTypesFilter(TargetTypesFilter),
    Grouping(Grouping),
}

#[derive(Debug)]
pub struct Ui {
    data: CommandData,
    settings: OutlineSettings,
    ui: CargoConfigurationTreeProvider,
    outline_ui: CargoOutlineTreeProvider,
    filtered_packages: Vec<CondensedPackage>,
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

        let settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();
        let config: Config = get_state_vs_code(state_key(&root_dir)).unwrap_or_default();

        let base = Base {
            cmds,
            file_watcher,
            root_dir,
        };

        let (ui_tx, ui_rx) = channel(CHANNEL_CAPACITY);
        let (outline_tx, outline_rx) = channel(CHANNEL_CAPACITY);
        let handler = CargoConfigurationTreeProviderHandler::new(ui_tx);
        let outline_handler = CargoOutlineTreeProviderHandler::new(outline_tx);

        let data = CommandData {
            metadata: UiMetadata::default(),
            config,
        };

        let this = Self {
            data,
            settings,
            ui: CargoConfigurationTreeProvider::new(handler),
            outline_ui: CargoOutlineTreeProvider::new(outline_handler),
            filtered_packages: Vec::new(),
            base,
            cmd_tx,
        };

        // manifest, ui and cmd updates will run for the lifetime of the extension
        let manifest_update = Task::stream(manifest_changed_rx).map(|()| Message::ManifestChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        let ui_config_request = Task::stream(ui_rx).map(Message::ConfigUiRequest);
        let ui_outline_request = Task::stream(outline_rx).map(Message::OutlineUiRequest);
        // metadata is to initially parse the Cargo.toml
        let metadata = this.parse_manifest();
        let tasks = Task::batch([
            manifest_update,
            cmd,
            ui_config_request,
            ui_outline_request,
            metadata,
        ]);

        (this, tasks)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectionChanged(update) => {
                self.data.config.update(update);
                self.update_ui();

                Task::future(persist_state_vs_code(
                    state_key(&self.base.root_dir),
                    self.data.config.clone(),
                ))
                .discard()
            }
            Message::MetadataChanged(update) => match update {
                MetadataUpdate::Metadata(metadata) => {
                    // Update file watcher
                    let mut manifests = workspace_manifests(&metadata);
                    manifests.push(self.root_manifest());
                    self.base.file_watcher.watch_files(manifests);

                    self.data.metadata.metadata = Some(metadata);
                    self.update_ui();

                    Task::future(set_cargo_context(true)).discard()
                }
                MetadataUpdate::Profiles(profiles) => {
                    self.data.metadata.profiles = profiles;
                    Task::none()
                }
                MetadataUpdate::NoCargoToml(_error) => {
                    // TODO: log error
                    // Always check for mainfest in root dir
                    self.base
                        .file_watcher
                        .watch_files(vec![self.root_manifest()]);

                    self.data.metadata = UiMetadata::default();
                    self.update_ui();

                    Task::future(set_cargo_context(false)).discard()
                }
                // For invalid makefiles leave everything as is
                MetadataUpdate::FailedToRetrieve => Task::none(),
            },
            Message::ManifestChanged => self.parse_manifest(),
            Message::SettingsChanged(update) => self.update_settings(update),
            Message::Cmd(cmd) => self.process_cmd(cmd),
            Message::ConfigUiRequest(request) => self.send_config_nodes(request),
            Message::OutlineUiRequest(request) => self.send_outline_nodes(request),
        }
    }

    fn update_settings(&mut self, update: SettingsUpdate) -> Task<Message> {
        let settings = &mut self.settings;
        match update {
            SettingsUpdate::PackageFilter(filter) => settings.package_filter = filter,
            SettingsUpdate::TargetTypesFilter(filter) => settings.target_types_filter = filter,
            SettingsUpdate::Grouping(grouping) => settings.grouping = grouping,
        }
        self.update_ui();

        Task::future(persist_state_vs_code(
            settings_key(&self.base.root_dir),
            self.settings.clone(),
        ))
        .discard()
    }

    fn update_ui(&mut self) {
        self.update_condensed_packages();
        self.ui.update();
        self.outline_ui.update();
    }

    fn parse_manifest(&self) -> Task<Message> {
        let metadata = Task::future(parse_metadata(self.root_manifest(), exec_vs_code));
        let profiles = Task::future(parse_profiles(
            self.base.root_dir.clone(),
            read_file_vs_code,
        ));
        Task::batch([metadata, profiles]).map(Message::MetadataChanged)
    }

    fn root_manifest(&self) -> String {
        format!("{}/Cargo.toml", self.base.root_dir)
    }

    fn send_config_nodes(&self, request: ConfigUiRequest) -> Task<Message> {
        let ConfigUiRequest { mut tx, node_type } = request;

        let config = self.data.config.clone();
        let available_features = self.data.available_features().unwrap_or_default();
        let nodes = node_type
            .map(|h| h.children(&config, &available_features))
            .unwrap_or(NodeData::roots());

        Task::future(async move { tx.send(nodes).await }).discard()
    }

    fn update_condensed_packages(&mut self) {
        if let Some(metadata) = &self.data.metadata.metadata {
            self.filtered_packages = self.settings.filter(metadata);
        }
    }

    fn send_outline_nodes(&self, request: OutlineUiRequest) -> Task<Message> {
        let OutlineUiRequest { mut tx, node_type } = request;

        // No metadata - nothing to show
        let Some(metadata) = &self.data.metadata.metadata else {
            return Task::none();
        };

        let Some(node_type) = node_type else {
            // Root node
            let Some(name) = Path::new(&self.base.root_dir)
                .file_name()
                .and_then(|n| n.to_str().map(|n| n.to_string()))
            else {
                return Task::none();
            };
            let num_packages = metadata.workspace_packages().len();
            let root = vec![OutlineNodeData::root(name, num_packages)];
            return Task::future(async move { tx.send(root).await }).discard();
        };

        let nodes = node_type.children(
            &self.data.config,
            &self.filtered_packages,
            self.settings.grouping,
            self.settings.target_types_filter.features,
        );
        Task::future(async move { tx.send(nodes).await }).discard()
    }
}

pub fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.cargo.ui_settings")
}

pub fn state_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.cargo.state")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OutlineSettings {
    pub package_filter: String,
    pub target_types_filter: TargetTypesFilter,
    pub grouping: Grouping,
}

impl OutlineSettings {
    fn filter(&self, metadata: &Metadata) -> Vec<CondensedPackage> {
        let package_filter = &self.package_filter;
        let target_types_filter = &self.target_types_filter;

        metadata
            .workspace_packages()
            .iter()
            // Filter by package name
            .filter(|pkg| package_filter.is_empty() || pkg.name.contains(package_filter))
            .sorted_by_key(|pkg| pkg.name.clone())
            .map(|pkg| CondensedPackage {
                name: pkg.name.to_string(),
                manifest: pkg.manifest_path.to_string(),
                targets: pkg
                    .targets
                    .iter()
                    // Filter by target type
                    .filter(|target| target_types_filter.keep(target))
                    .filter_map(CondensedTarget::try_from_cargo)
                    .sorted_by_key(|t| t.target_type)
                    .collect(),
                features: iter::once("All features".to_string())
                    .chain(pkg.features.keys().cloned())
                    .collect(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetTypesFilter {
    bin: bool,
    lib: bool,
    example: bool,
    benchmarks: bool,
    features: bool,
}

impl Default for TargetTypesFilter {
    fn default() -> Self {
        Self {
            bin: true,
            lib: true,
            example: true,
            benchmarks: true,
            features: true,
        }
    }
}

impl TargetTypesFilter {
    fn keep(&self, target: &cargo_metadata::Target) -> bool {
        match Target::from_target(target) {
            Some(Target::Bin) => self.bin,
            Some(Target::Lib) => self.lib,
            Some(Target::Example) => self.example,
            Some(Target::Bench) => self.benchmarks,
            None => false,
        }
    }

    fn all_filtered() -> Self {
        Self {
            bin: false,
            lib: false,
            example: false,
            benchmarks: false,
            features: false,
        }
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
    config: Config,
}

impl CommandData {
    fn profiles(&self) -> SelectInput<Profile> {
        let options = self.metadata.profiles().to_vec();
        let current = vec![self.config.profile.clone()];

        SelectInput { options, current }
    }

    fn packages(&self) -> SelectInput<Option<String>> {
        let options = self.metadata.package_options();
        let current = vec![self.config.selected_package.clone()];

        SelectInput { options, current }
    }

    fn build_target_options(&self) -> Option<SelectInput<Option<BuildSubTarget>>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let config = &self.config;

        let options = config.build_target_options(metadata);
        let current = vec![
            config
                .package_selection()
                .and_then(|s| s.build_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    fn run_target_options(&self) -> Option<SelectInput<Option<RunSubTarget>>> {
        let metadata = self.metadata.metadata.as_ref()?;

        let config = &self.config;

        let options = config.run_target_options(metadata);
        let current = vec![
            config
                .package_selection()
                .and_then(|s| s.run_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    fn bench_target_options(&self) -> Option<SelectInput<Option<String>>> {
        let metadata = self.metadata.metadata.as_ref()?;
        let config = &self.config;

        let options = config.bench_target_options(metadata);
        let current = vec![
            config
                .package_selection()
                .and_then(|s| s.benchmark_target.clone()),
        ];

        Some(SelectInput { options, current })
    }

    fn available_features(&self) -> Option<Vec<String>> {
        let metadata = self.metadata.metadata.as_ref()?;

        Some(self.config.feature_options(metadata))
    }

    fn feature_options(&self) -> Option<SelectInput<String>> {
        let options = self.available_features()?;
        let current = match self.config.selected_features() {
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
