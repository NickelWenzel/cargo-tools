use std::{
    iter,
    path::{Path, PathBuf},
};

use cargo_tools::{
    CargoCommand,
    cargo::{
        Config, ConfigUpdate, Features, Profile,
        command::{BuildSubTarget, BuildTarget, RunSubTarget, RunTarget},
        config::{self, FeatureTarget},
        metadata::{Metadata, Package, Target, TargetType},
    },
};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_viewless::Task;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{js_sys::Array, spawn_local};

use crate::{
    environment::CommandExt,
    extension::{
        CommandBinding,
        workspace::outline::{
            command::{Command, register_outline_commands},
            treeprovider::{CargoOutlineTreeProviderHandler, OutlineNodeData, OutlineUiRequest},
        },
    },
    quick_pick::show_quick_pick_type,
    quick_pick::{SelectInput, ToQuickPickItem},
    runtime::{CHANNEL_CAPACITY, VsCodeTask, get_state_vs_code, persist_state_vs_code},
    runtime::{JsValueExt, debug, execute_task, host_platform},
};
use tracing::{error, info};

#[wasm_bindgen::prelude::wasm_bindgen(raw_module = "../outlineTreeProvider.ts")]
extern "C" {
    type CargoOutlineTreeProvider;

    #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
    fn new(handler: CargoOutlineTreeProviderHandler) -> CargoOutlineTreeProvider;

    #[wasm_bindgen::prelude::wasm_bindgen(method)]
    fn update(this: &CargoOutlineTreeProvider);
}

#[derive(Debug)]
pub enum Message {
    MetadataChanged,
    ConfigChanged,
    SettingsChanged(SettingsUpdate),
    Cmd(Command),
    OutlineUiRequest(OutlineUiRequest),
}

pub enum Event {
    ConfigUpdate(config::Update),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    PackageFilter(String),
    TargetTypesFilter(TargetTypesFilter),
    Grouping(Grouping),
}

pub struct Outline {
    settings: Settings,
    ui: CargoOutlineTreeProvider,
    filtered_packages: Vec<Package>,
    _cmds: Vec<CommandBinding>,
    root_dir: String,
    cmd_tx: Sender<Command>,
}

impl Outline {
    pub fn init(root_dir: String) -> (Outline, Task<Message>) {
        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_outline_commands(cmd_tx.clone());

        let settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let (outline_tx, outline_rx) = channel(CHANNEL_CAPACITY);
        let outline_handler = CargoOutlineTreeProviderHandler::new(outline_tx);

        let this = Self {
            settings,
            ui: CargoOutlineTreeProvider::new(outline_handler),
            filtered_packages: Vec::new(),
            _cmds,
            root_dir,
            cmd_tx,
        };

        // ui and cmd updates will run for the lifetime of the extension
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        let ui_request = Task::stream(outline_rx).map(Message::OutlineUiRequest);
        let tasks = Task::batch([cmd, ui_request]);

        (this, tasks)
    }

    pub fn update(
        &mut self,
        msg: Message,
        metadata: &Metadata,
        config: &Config,
    ) -> (Task<Message>, Option<Event>) {
        match msg {
            Message::MetadataChanged => {
                let packages = metadata.packages();
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

                self.ui.update();
                (Task::none(), None)
            }
            Message::ConfigChanged => {
                self.ui.update();
                (Task::none(), None)
            }
            Message::SettingsChanged(update) => {
                let settings = &mut self.settings;
                match update {
                    SettingsUpdate::PackageFilter(filter) => settings.package_filter = filter,
                    SettingsUpdate::TargetTypesFilter(filter) => {
                        settings.target_types_filter = filter
                    }
                    SettingsUpdate::Grouping(grouping) => settings.grouping = grouping,
                }
                self.ui.update();

                let task = Task::future(persist_state_vs_code(
                    settings_key(&self.root_dir),
                    self.settings.clone(),
                ))
                .discard();
                (task, None)
            }
            Message::Cmd(cmd) => self.handle_cmd(cmd, metadata, config),
            Message::OutlineUiRequest(request) => {
                let OutlineUiRequest { mut tx, node_type } = request;

                let Some(node_type) = node_type else {
                    // Root node
                    let Some(name) = Path::new(&self.root_dir)
                        .file_name()
                        .and_then(|n| n.to_str().map(|n| n.to_string()))
                    else {
                        return (Task::none(), None);
                    };
                    let num_packages = metadata.packages().len();
                    let root = vec![OutlineNodeData::root(name, num_packages)];
                    return (
                        Task::future(async move { tx.send(root).await }).discard(),
                        None,
                    );
                };

                let nodes = node_type.children(
                    config,
                    &self.filtered_packages,
                    self.settings.grouping,
                    self.settings.target_types_filter.features,
                );
                let task = Task::future(async move { tx.send(nodes).await }).discard();
                (task, None)
            }
        }
    }

    fn handle_cmd(
        &self,
        cmd: Command,
        metadata: &Metadata,
        config: &Config,
    ) -> (Task<Message>, Option<Event>) {
        match cmd {
            Command::Select(update) => (Task::none(), Some(Event::ConfigUpdate(update))),
            Command::Unselect(update) => (Task::none(), Some(Event::ConfigUpdate(update))),
            Command::Build(target) => (self.cmd_exec(CargoCommand::Build(target), config), None),
            Command::Test(package) => (self.cmd_exec(CargoCommand::Test { package }, config), None),
            Command::Clean(package) => {
                (self.cmd_exec(CargoCommand::Clean { package }, config), None)
            }
            Command::Run(target) => (self.cmd_exec(CargoCommand::Run(Some(target)), config), None),
            Command::Debug(target) => (self.debug(target, metadata, config.clone()), None),
            Command::Bench(target) => (
                self.cmd_exec(CargoCommand::Bench(Some(target)), config),
                None,
            ),
            Command::SelectWorkspaceMemberFilter => {
                (self.select_workspace_member_filter(metadata), None)
            }
            Command::EditWorkspaceMemberFilter(filter) => {
                let task = Task::done(Message::SettingsChanged(SettingsUpdate::PackageFilter(
                    filter,
                )));
                (task, None)
            }
            Command::SelectTargetTypeFilter => (self.select_target_type_filter(), None),
            Command::EditTargetTypeFilter(update) => (Task::done(update.into_cargo_msg()), None),
            Command::ClearAllFilters => {
                let member_filter = Task::done(Message::SettingsChanged(
                    SettingsUpdate::PackageFilter(String::new()),
                ));
                let types_filter = Task::done(TargetTypesFilter::default().into_cargo_msg());

                (Task::batch([member_filter, types_filter]), None)
            }
            Command::ToggleWorkspaceMemberGrouping => (
                Task::done(self.settings.grouping.toggle().into_cargo_msg()),
                None,
            ),
            Command::ToggleFeature {
                feature_type,
                feature,
            } => {
                let event = self.toggle_feature(config, feature_type, feature);
                (Task::none(), Some(event))
            }
        }
    }

    fn cmd_exec(&self, cmd: CargoCommand, config: &Config) -> Task<Message> {
        let ctx = cmd.ctx();

        match cmd.try_into_process(config, ctx) {
            Ok(process) => Task::future(execute_task(VsCodeTask::cargo(process))).discard(),
            Err(e) => {
                error!("{e}");
                Task::none()
            }
        }
    }

    fn debug(&self, target: RunTarget, metadata: &Metadata, mut config: Config) -> Task<Message> {
        let Some(run_target) = target.target.as_ref() else {
            return Task::none();
        };
        let build_sub_target = match run_target {
            RunSubTarget::Bin(t) => BuildSubTarget::Bin(t.clone()),
            RunSubTarget::Example(t) => BuildSubTarget::Example(t.clone()),
        };
        let build_target = BuildTarget {
            package: target.package.clone(),
            target: Some(build_sub_target),
        };

        config.profile = Profile::Dev; // For now always use standard dev profile

        let build_debug_cmd = CargoCommand::Build(Some(build_target));
        let ctx = build_debug_cmd.ctx();

        let build_debug_process = match build_debug_cmd.try_into_process(&config, ctx) {
            Ok(process) => process,
            Err(e) => {
                error!("{e}");
                return Task::none();
            }
        };

        let target_exe_path = exec_path(run_target, &config, metadata.target_dir());

        Task::future(async move {
            execute_task(VsCodeTask::cargo(build_debug_process)).await;

            if let Err(e) = debug(&target_exe_path, &target.package).await {
                error!("Error while debugging: {}", e.to_error_string());
            }
        })
        .discard()
    }

    fn select_workspace_member_filter(&self, metadata: &Metadata) -> Task<Message> {
        let current = self.settings.package_filter.clone();
        let Ok(options) = metadata
            .packages()
            .iter()
            .map(|p| to_value(&p.name.to_string().to_item(false)))
            .collect::<Result<Array, _>>()
        else {
            return Task::none();
        };

        if options.length() < 2 {
            return Task::none();
        }

        let cmd_tx = self.cmd_tx.clone();

        Task::future(async move {
            // Closure only needs to live while the quickpick is active
            let filter_update = Closure::new(move |filter: String| {
                let mut tx = cmd_tx.clone();
                spawn_local(async move {
                    info!("Sending workspace member filter '{filter}'");
                    if let Err(e) = tx.send(Command::EditWorkspaceMemberFilter(filter)).await {
                        error!("Failed to queue msg: {}", e);
                    }
                });
            });

            let filter = show_quick_pick_type(current.clone(), options, &filter_update)
                .await
                .map(|f| f.as_string().unwrap_or(current.clone()))
                .unwrap_or(current);

            Message::SettingsChanged(SettingsUpdate::PackageFilter(filter))
        })
    }

    fn select_target_type_filter(&self) -> Task<Message> {
        let categories: Vec<_> = [
            "Libraries",
            "Binaries",
            "Examples",
            "Benchmarks",
            "Features",
        ]
        .map(str::to_string)
        .into_iter()
        .collect();

        let selected = self.settings.target_types_filter.to_selected();
        let current = self.settings.target_types_filter.clone();

        let input = SelectInput {
            options: categories,
            current: selected,
        };

        let cmd_tx = self.cmd_tx.clone();
        let filter_update = move |selected: Vec<String>| {
            info!("Received category filter update from quickpick'{selected:?}'");
            let mut tx = cmd_tx.clone();
            spawn_local(async move {
                let filter = TargetTypesFilter::from_selected(selected);

                if let Err(e) = tx.send(Command::EditTargetTypeFilter(filter)).await {
                    error!("Failed to queue msg: {}", e);
                }
            });
        };

        Task::future(async move {
            let selected_categories = input
                .select_multiple(filter_update)
                .await
                .map(TargetTypesFilter::from_selected)
                .unwrap_or(current);

            SettingsUpdate::TargetTypesFilter(selected_categories)
        })
        .map(Message::SettingsChanged)
    }

    fn toggle_feature(
        &self,
        config: &Config,
        feature_type: FeatureTarget,
        feature: String,
    ) -> Event {
        let selected_features = match &feature_type {
            FeatureTarget::Package(package) => config
                .get(package, |s| Some(s.selected_features.clone()))
                .unwrap_or_default(), // get current feature or fall back to empty selection
            FeatureTarget::Workspace => config.selected_features.clone(),
        };

        let features = match selected_features {
            Features::All => Features::Some(Vec::from_iter(
                (&feature != "All features").then_some(feature),
            )),
            Features::Some(mut features) => {
                if &feature == "All features" {
                    Features::All
                } else {
                    if let Some(pos) = features.iter().position(|x| *x == feature) {
                        features.remove(pos);
                    } else {
                        features.push(feature);
                    }
                    Features::Some(features)
                }
            }
        };

        Event::ConfigUpdate(ConfigUpdate::SelectedFeatures {
            feature_target: feature_type,
            features,
        })
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
    fn from_selected(selected: Vec<String>) -> Self {
        let mut filter = Self::all_filtered();
        if selected.contains(&"Libraries".to_string()) {
            filter.lib = true;
        }
        if selected.contains(&"Binaries".to_string()) {
            filter.bin = true;
        }
        if selected.contains(&"Examples".to_string()) {
            filter.example = true;
        }
        if selected.contains(&"Benchmarks".to_string()) {
            filter.benchmarks = true;
        }
        if selected.contains(&"Features".to_string()) {
            filter.features = true;
        }
        filter
    }

    fn to_selected(&self) -> Vec<String> {
        let mut selected = Vec::new();

        if self.lib {
            selected.push("Libraries".to_string());
        }

        if self.bin {
            selected.push("Binaries".to_string());
        }

        if self.example {
            selected.push("Examples".to_string());
        }

        if self.benchmarks {
            selected.push("Benchmarks".to_string());
        }

        if self.features {
            selected.push("Features".to_string());
        }
        selected
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    package_filter: String,
    target_types_filter: TargetTypesFilter,
    grouping: Grouping,
}

impl Settings {
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

pub fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.workspace.outline.settings")
}

fn exec_path(target: &RunSubTarget, config: &Config, target_dir: &str) -> String {
    let path_components = iter::once(target_dir.to_string())
        .chain(iter::once("debug".to_string())) // For now always assume debug profile
        .chain(config.platform_target.as_ref().map(|t| t.to_string()))
        .chain(match target {
            RunSubTarget::Bin(bin) => vec![bin.clone()],
            RunSubTarget::Example(example) => {
                vec!["examples".to_string(), example.to_string()]
            }
        })
        .chain((host_platform() == "windows").then_some(".exe".to_string()));

    PathBuf::from_iter(path_components)
        .to_string_lossy()
        .to_string()
}

trait IntoMessage {
    fn into_cargo_msg(self) -> Message;
}

impl IntoMessage for TargetTypesFilter {
    fn into_cargo_msg(self) -> Message {
        Message::SettingsChanged(SettingsUpdate::TargetTypesFilter(self))
    }
}

impl IntoMessage for Grouping {
    fn into_cargo_msg(self) -> Message {
        Message::SettingsChanged(SettingsUpdate::Grouping(self))
    }
}
