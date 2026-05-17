use std::{collections::HashMap, iter, path::PathBuf};

use cargo_tools::{
    CargoCommand,
    cargo::{
        Config, ConfigUpdate, Features, Profile,
        command::{BenchTarget, BuildSubTarget, BuildTarget, RunSubTarget, RunTarget},
        config::FeatureTarget,
        metadata::Metadata,
    },
    process::Process,
};
use futures::{SinkExt, channel::mpsc::channel};
use iced_viewless::Task;

use crate::{
    environment::CommandExt,
    extension::{
        CommandBinding,
        workspace::configuration::{
            command::{Command, register_configuration_commands},
            treeprovider::{CargoConfigurationTreeProviderHandler, ConfigUiRequest, NodeData},
        },
    },
    quick_pick::SelectInput,
    runtime::{
        CHANNEL_CAPACITY, VsCodeTask, exec_vs_code, get_state_vs_code, persist_state_vs_code,
    },
    vs_code_api::{
        CargoConfigurationTreeProvider, JsValueExt, debug, execute_task,
        get_rust_analyzer_check_targets, host_platform, log_error,
        update_rust_analyzer_check_targets,
    },
};

#[derive(Debug)]
pub enum Message {
    MetadataChanged,
    ConfigChanged(ConfigUpdate),
    Cmd(Command),
    ConfigUiRequest(ConfigUiRequest),
}

pub enum Event {
    ConfigUpdate,
}

pub struct Configuration {
    config: Config,
    ui: CargoConfigurationTreeProvider,
    _cmds: Vec<CommandBinding>,
    root_dir: String,
}

impl Configuration {
    pub fn init(root_dir: String) -> (Configuration, Task<Message>) {
        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_configuration_commands(cmd_tx);

        let config: Config = get_state_vs_code(state_key(&root_dir)).unwrap_or_default();

        let (ui_tx, ui_rx) = channel(CHANNEL_CAPACITY);
        let handler = CargoConfigurationTreeProviderHandler::new(ui_tx);

        let this = Self {
            config,
            ui: CargoConfigurationTreeProvider::new(handler),
            _cmds,
            root_dir,
        };

        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        let ui_config_request = Task::stream(ui_rx).map(Message::ConfigUiRequest);
        let tasks = Task::batch([cmd, ui_config_request]);

        (this, tasks)
    }

    pub fn update(&mut self, msg: Message, metadata: &Metadata) -> (Task<Message>, Option<Event>) {
        match msg {
            Message::MetadataChanged => {
                self.ui.update();
                (Task::none(), None)
            }
            Message::ConfigChanged(update) => {
                self.config.update(update);
                self.ui.update();

                let task = Task::future(persist_state_vs_code(
                    state_key(&self.root_dir),
                    self.config.clone(),
                ))
                .discard();
                (task, Some(Event::ConfigUpdate))
            }
            Message::Cmd(cmd) => (self.handle_cmd(cmd, metadata), None),
            Message::ConfigUiRequest(request) => {
                let ConfigUiRequest { mut tx, node_type } = request;

                let config = self.config.clone();
                let available_features = self.config.feature_options(metadata);
                let nodes = node_type
                    .map(|node| node.children(&config, &available_features))
                    .unwrap_or(NodeData::roots());

                (
                    Task::future(async move { tx.send(nodes).await }).discard(),
                    None,
                )
            }
        }
    }

    fn handle_cmd(&self, cmd: Command, metadata: &Metadata) -> Task<Message> {
        match cmd {
            Command::SelectProfile => {
                let options = metadata.profiles().to_vec();
                let current = vec![self.config.profile.clone()];

                let input = SelectInput { options, current };
                done(async move { input.select().await.map(ConfigUpdate::SelectedProfile) })
            }
            Command::SelectPackage => {
                let options = iter::once(None)
                    .chain(metadata.packages().iter().map(|p| Some(p.name.clone())))
                    .collect();
                let current = vec![self.config.selected_package.clone()];

                let input = SelectInput { options, current };
                done(async move { input.select().await.map(ConfigUpdate::SelectedPackage) })
            }
            Command::SelectBuildTarget => {
                let options = self.config.build_target_options(metadata);
                let current = vec![
                    self.config
                        .package_selection()
                        .and_then(|s| s.build_target.clone()),
                ];

                let input = Some(SelectInput { options, current });
                done(async move { input?.select().await.map(ConfigUpdate::SelectedBuildTarget) })
            }
            Command::SelectRunTarget => {
                let options = self.config.run_target_options(metadata);
                let current = vec![
                    self.config
                        .package_selection()
                        .and_then(|s| s.run_target.clone()),
                ];

                let input = Some(SelectInput { options, current });
                done(async move { input?.select().await.map(ConfigUpdate::SelectedRunTarget) })
            }
            Command::SelectBenchmarkTarget => {
                let options = self.config.bench_target_options(metadata);
                let current = vec![
                    self.config
                        .package_selection()
                        .and_then(|s| s.benchmark_target.clone()),
                ];

                let input = Some(SelectInput { options, current });
                done(async move {
                    input?
                        .select()
                        .await
                        .map(ConfigUpdate::SelectedBenchmarkTarget)
                })
            }
            Command::SelectPlatformTarget => {
                let current = self.config.platform_target.clone();
                done(async move { select_platform_target(current.clone()).await })
            }
            Command::InstallPlatformTarget => Task::future(install_platform_target()).discard(),
            Command::SetRustAnalyzerCheckTargets => {
                Task::future(set_rust_analyzer_check_targets()).discard()
            }
            Command::BuildDocs => self.cmd_exec(CargoCommand::Doc),
            Command::SelectFeatures => {
                let options = self.config.feature_options(metadata);
                let current = match self.config.selected_features() {
                    Features::All => ["All features".to_string()].to_vec(),
                    Features::Some(features) => features,
                };

                let input = Some(SelectInput { options, current });
                let feature_target = self.config.feature_target();
                done(async move { select_features(input, feature_target).await })
            }
            Command::Refresh => self.refresh(metadata),
            Command::Clean => {
                let package = self.config.selected_package.clone();
                self.cmd_exec(CargoCommand::Clean { package })
            }
            Command::Build => {
                let target = self.config.selected_package.clone().map(|package| {
                    let target = self.config.get(&package, |s| s.build_target.clone());
                    BuildTarget { package, target }
                });
                self.cmd_exec(CargoCommand::Build(target))
            }
            Command::Run => {
                let target = self.config.selected_package.clone().map(|package| {
                    let target = self.config.get(&package, |s| s.run_target.clone());
                    RunTarget { package, target }
                });
                self.cmd_exec(CargoCommand::Run(target))
            }
            Command::Debug => match self.config.selected_package.clone() {
                Some(package) => {
                    let target = self.config.get(&package, |s| s.run_target.clone());
                    self.debug(RunTarget { package, target }, metadata.target_dir())
                }
                None => Task::none(),
            },
            Command::Test => {
                let package = self.config.selected_package.clone();
                self.cmd_exec(CargoCommand::Test { package })
            }
            Command::Bench => {
                let target = self.config.selected_package.clone().map(|package| {
                    let target = self.config.get(&package, |s| s.benchmark_target.clone());
                    BenchTarget { package, target }
                });
                self.cmd_exec(CargoCommand::Bench(target))
            }
            Command::ToggleFeature(feature) => {
                let feature_target = self.config.feature_target();
                self.toggle_feature(feature_target, feature)
            }
        }
    }

    fn cmd_exec(&self, cmd: CargoCommand) -> Task<Message> {
        let ctx = cmd.ctx();

        match cmd.try_into_process(&self.config, ctx) {
            Ok(process) => Task::future(execute_task(VsCodeTask::cargo(process))).discard(),
            Err(e) => {
                log_error(&e.to_string());
                Task::none()
            }
        }
    }

    fn debug(&self, target: RunTarget, target_dir: &str) -> Task<Message> {
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
        let mut config = self.config.clone();
        config.profile = Profile::Dev; // For now always use standard dev profile

        let build_debug_cmd = CargoCommand::Build(Some(build_target));
        let ctx = build_debug_cmd.ctx();

        let build_debug_process = match build_debug_cmd.try_into_process(&config, ctx) {
            Ok(process) => process,
            Err(e) => {
                log_error(&e.to_string());
                return Task::none();
            }
        };

        let target_exe_path = exec_path(run_target, &self.config, target_dir);

        Task::future(async move {
            execute_task(VsCodeTask::cargo(build_debug_process)).await;

            if let Err(e) = debug(&target_exe_path, &target.package).await {
                log_error(&format!("Error while debugging: {}", e.to_error_string()));
            }
        })
        .discard()
    }

    fn toggle_feature(&self, feature_type: FeatureTarget, feature: String) -> Task<Message> {
        let selected_features = match &feature_type {
            FeatureTarget::Package(package) => self
                .config
                .get(package, |s| Some(s.selected_features.clone()))
                .unwrap_or_default(), // get current feature or fall back to empty selection
            FeatureTarget::Workspace => self.config.selected_features.clone(),
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

        Task::done(
            ConfigUpdate::SelectedFeatures {
                feature_target: feature_type,
                features,
            }
            .into_cargo_msg(),
        )
    }

    fn refresh(&self, metadata: &Metadata) -> Task<Message> {
        // Weed out packages that do not exist anymore except for current selection
        let package_selection = self
            .config
            .package_configs
            .iter()
            .filter(|(package, _)| {
                let is_selected = self
                    .config
                    .selected_package
                    .as_ref()
                    .is_some_and(|p| &p == package);
                let is_in_metadata = metadata
                    .packages()
                    .iter()
                    .any(|p| &p.name.as_str() == package);
                is_selected || is_in_metadata
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Task::done(ConfigUpdate::Refresh(package_selection).into_cargo_msg())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

fn state_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.workspace.configuration")
}

fn done(fut: impl Future<Output = Option<ConfigUpdate>> + 'static) -> Task<Message> {
    Task::future(fut)
        .and_then(Task::done)
        .map(IntoMessage::into_cargo_msg)
}

async fn select_platform_target(current: Option<String>) -> Option<ConfigUpdate> {
    let options = platform_targets().await.map(|targets| {
        targets
            .into_iter()
            .filter(|t| t.ends_with("(installed)"))
            .map(|t| t.trim_end_matches("(installed)").trim().to_string())
            .map(Some)
    })?;

    let input = {
        SelectInput {
            options: iter::once(None).chain(options).collect(),
            current: vec![current],
        }
    };

    input
        .select()
        .await
        .map(ConfigUpdate::SelectedPlatformTarget)
}

trait IntoMessage {
    fn into_cargo_msg(self) -> Message;
}

impl IntoMessage for ConfigUpdate {
    fn into_cargo_msg(self) -> Message {
        Message::ConfigChanged(self)
    }
}

async fn platform_targets() -> Option<Vec<String>> {
    let process = Process::new(
        "rustup".to_string(),
        vec!["target".to_string(), "list".to_string()],
        HashMap::new(),
    );
    match exec_vs_code(process).await {
        Ok(output) => Some(output.lines().map(|l| l.trim().to_string()).collect()),
        Err(e) => {
            log_error(&format!("Failed to get platform targets from rustup: {e}"));
            None
        }
    }
}

async fn install_platform_target() {
    let options = match platform_targets().await {
        Some(targets) => targets
            .into_iter()
            .filter(|t| !t.ends_with("(installed)"))
            .collect(),
        None => return,
    };

    let input = SelectInput {
        options,
        current: Vec::new(),
    };

    let Some(target) = input.select().await else {
        return;
    };

    execute_task(VsCodeTask::rustup(Process::new(
        "rustup".to_string(),
        vec!["target".to_string(), "add".to_string(), target],
        HashMap::new(),
    )))
    .await
}

async fn set_rust_analyzer_check_targets() {
    let current = get_rust_analyzer_check_targets();
    let options = match platform_targets().await {
        Some(targets) => targets
            .into_iter()
            .filter(|t| t.ends_with("(installed)"))
            .map(|t| t.trim_end_matches("(installed)").trim().to_string())
            .collect(),
        None => return,
    };

    let input = SelectInput { options, current };

    if let Some(targets) = input.select_multiple(|_| {}).await {
        update_rust_analyzer_check_targets(targets).await;
    }
}

async fn select_features(
    input: Option<SelectInput<String>>,
    feature_target: FeatureTarget,
) -> Option<ConfigUpdate> {
    let selected_features = input?.select_multiple(|_| {}).await?;
    let features = if selected_features.iter().any(|f| f == "All features") {
        Features::All
    } else {
        Features::Some(selected_features)
    };

    Some(ConfigUpdate::SelectedFeatures {
        feature_target,
        features,
    })
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
