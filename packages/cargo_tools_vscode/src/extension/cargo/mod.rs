pub mod command;
pub mod ui;

use std::{iter, path::Path};

use cargo_tools::cargo::{
    Config, ConfigUpdate, Profile,
    command::{BuildSubTarget, RunSubTarget},
    config::Features,
    metadata::{Metadata, Package, ParseError, Target, TargetType, parse_metadata},
};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_viewless::Task;

use serde::{Deserialize, Serialize};

use crate::{
    environment::metadata_task_context,
    extension::{
        CommandBinding,
        cargo::{
            command::{Command, register_cargo_commands},
            ui::{
                CargoConfigurationTreeProviderHandler, CargoOutlineTreeProviderHandler,
                ConfigUiRequest, NodeData, OutlineNodeData, OutlineUiRequest,
            },
        },
        send_file_changed,
    },
    quick_pick::SelectInput,
    runtime::{
        CHANNEL_CAPACITY, exec_vs_code, get_state_vs_code, persist_state_vs_code, read_file_vs_code,
    },
    vs_code_api::{
        CargoConfigurationTreeProvider, CargoOutlineTreeProvider, TsFileWatcher, log_error,
        set_cargo_context,
    },
};

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    Metadata(Metadata),
    NoCargoToml(String),
    FailedToParse(String),
    CargoCommandEmpty(String),
}

impl MetadataUpdate {
    fn from_parse_result(res: Result<Metadata, ParseError>) -> Self {
        match res {
            Ok(metadata) => Self::Metadata(metadata),
            Err(e) => match e {
                ParseError::NoCargoToml(e) => Self::NoCargoToml(e),
                ParseError::Parse(e) => Self::FailedToParse(e.to_string()),
                ParseError::CargoCommandEmpty(e) => Self::CargoCommandEmpty(e.to_string()),
            },
        }
    }
}

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
    filtered_packages: Vec<Package>,
    _cmds: Vec<CommandBinding>,
    file_watcher: TsFileWatcher,
    root_dir: String,
    cmd_tx: Sender<Command>,
}

impl Ui {
    /// Inits all data and update channels
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        // Init manifest file updates
        let (manifest_changed_tx, manifest_changed_rx) = channel(CHANNEL_CAPACITY);
        let file_watcher = TsFileWatcher::new(send_file_changed(manifest_changed_tx));

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_cargo_commands(cmd_tx.clone());

        let settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();
        let config: Config = get_state_vs_code(state_key(&root_dir)).unwrap_or_default();

        let (ui_tx, ui_rx) = channel(CHANNEL_CAPACITY);
        let (outline_tx, outline_rx) = channel(CHANNEL_CAPACITY);
        let handler = CargoConfigurationTreeProviderHandler::new(ui_tx);
        let outline_handler = CargoOutlineTreeProviderHandler::new(outline_tx);

        let data = CommandData {
            metadata: Metadata::default(),
            config,
        };

        let this = Self {
            data,
            settings,
            ui: CargoConfigurationTreeProvider::new(handler),
            outline_ui: CargoOutlineTreeProvider::new(outline_handler),
            filtered_packages: Vec::new(),
            _cmds,
            file_watcher,
            root_dir,
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
                    state_key(&self.root_dir),
                    self.data.config.clone(),
                ))
                .discard()
            }
            Message::MetadataChanged(update) => match update {
                MetadataUpdate::Metadata(metadata) => {
                    // Update file watcher
                    let mut manifests = metadata.manifests();
                    manifests.push(self.root_manifest());
                    self.file_watcher.watch_files(manifests);

                    self.data.metadata = metadata;
                    self.update_ui();

                    Task::future(set_cargo_context(true)).discard()
                }
                MetadataUpdate::NoCargoToml(e) => {
                    log_error(&e);
                    // Always check for mainfest in root dir
                    self.file_watcher.watch_files(vec![self.root_manifest()]);

                    self.data.metadata = Metadata::default();
                    self.update_ui();

                    Task::future(set_cargo_context(false)).discard()
                }
                // For invalid metadata or cargo command leave everything as is
                MetadataUpdate::CargoCommandEmpty(e) | MetadataUpdate::FailedToParse(e) => {
                    log_error(&e);
                    Task::none()
                }
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
            settings_key(&self.root_dir),
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
        Task::future(parse_metadata(
            self.root_dir.clone(),
            metadata_task_context(),
            exec_vs_code,
            read_file_vs_code,
        ))
        .map(MetadataUpdate::from_parse_result)
        .map(Message::MetadataChanged)
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
        let packages = self.data.metadata.packages();
        let filtered_packages = self.settings.filter_packages(packages);
        self.filtered_packages = filtered_packages
            .map(
                |Package {
                     name,
                     manifest,
                     targets,
                     features,
                 }| {
                    Package {
                        name: name.clone(),
                        manifest: manifest.clone(),
                        targets: self.settings.filter_targets(targets).cloned().collect(),
                        features: features.clone(),
                    }
                },
            )
            .collect();
    }

    fn send_outline_nodes(&self, request: OutlineUiRequest) -> Task<Message> {
        let OutlineUiRequest { mut tx, node_type } = request;

        // No metadata - nothing to show
        let metadata = &self.data.metadata;

        let Some(node_type) = node_type else {
            // Root node
            let Some(name) = Path::new(&self.root_dir)
                .file_name()
                .and_then(|n| n.to_str().map(|n| n.to_string()))
            else {
                return Task::none();
            };
            let num_packages = metadata.packages().len();
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

    fn root_manifest(&self) -> String {
        format!("{}/Cargo.toml", self.root_dir)
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
    package_filter: String,
    target_types_filter: TargetTypesFilter,
    grouping: Grouping,
}

impl OutlineSettings {
    fn filter_packages<'a>(&'a self, packages: &'a [Package]) -> impl Iterator<Item = &'a Package> {
        let allow_all = self.package_filter.is_empty();
        packages
            .iter()
            // Filter by package name
            .filter(move |pkg| allow_all || pkg.name.contains(&self.package_filter))
    }

    fn filter_targets<'a>(&'a self, targets: &'a [Target]) -> impl Iterator<Item = &'a Target> {
        targets
            .iter()
            // Filter by target type
            .filter(|target| self.target_types_filter.keep(target))
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
    fn keep(&self, target: &Target) -> bool {
        match target.target_type {
            TargetType::Bin => self.bin,
            TargetType::Lib => self.lib,
            TargetType::Example => self.example,
            TargetType::Bench => self.benchmarks,
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
    metadata: Metadata,
    config: Config,
}

impl CommandData {
    fn profiles(&self) -> SelectInput<Profile> {
        let options = self.metadata.profiles().to_vec();
        let current = vec![self.config.profile.clone()];

        SelectInput { options, current }
    }

    fn packages(&self) -> SelectInput<Option<String>> {
        let options = iter::once(None)
            .chain(
                self.metadata
                    .packages()
                    .iter()
                    .map(|p| Some(p.name.clone())),
            )
            .collect();
        let current = vec![self.config.selected_package.clone()];

        SelectInput { options, current }
    }

    fn build_target_options(&self) -> Option<SelectInput<Option<BuildSubTarget>>> {
        let metadata = &self.metadata;
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
        let metadata = &self.metadata;
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
        let metadata = &self.metadata;
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
        let metadata = &self.metadata;

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
