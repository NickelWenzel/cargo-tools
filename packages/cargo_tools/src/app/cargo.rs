pub mod command;
pub mod metadata;
pub mod selection;
pub mod ui;

use std::{collections::HashMap, iter};

use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{
    app::cargo::metadata::{MetadataUpdate, parse_manual, parse_metadata, workspace_manifests},
    configuration::{self, Configuration},
    runtime::{self, CargoTask, Runtime},
};

use ui::Message as UiMsg;

#[derive(Debug, Clone)]
pub enum CargoMessage<Ui: ui::Ui> {
    RootDirUpdate(String),
    ManifestUpdate,
    ConfigUpdate,
    MetadataUpdate(MetadataUpdate),
    Ui(UiMsg<Ui::CustomUpdate>),
}

use CargoMessage as Msg;

pub struct Cargo<Ui: ui::Ui> {
    root_dir: String,
    workspace_manifests: Vec<String>,
    ui: Ui,
    state: ui::State,
}

impl<Ui: ui::Ui> Cargo<Ui> {
    pub fn new(root_dir: String, ui: Ui) -> Self {
        Self {
            root_dir,
            workspace_manifests: Vec::new(),
            ui,
            state: Default::default(),
        }
    }
}

impl<Ui: ui::Ui + 'static> Cargo<Ui> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg<Ui>) -> Task<Msg<Ui>> {
        RT::log("Cargo update received".to_string());
        match msg {
            Msg::RootDirUpdate(root_dir) => {
                RT::log(format!("Cargo update received: new root dir {root_dir}"));
                self.update_root_dir::<RT>(root_dir)
            }
            Msg::ManifestUpdate => {
                RT::log("Cargo update received: manifest updated".to_string());
                self.parse::<RT>()
            }
            Msg::ConfigUpdate => {
                RT::log("Cargo update received: config updated".to_string());
                self.parse::<RT>()
            }
            Msg::MetadataUpdate(update) => {
                RT::log("Cargo update received: metadata updated".to_string());
                self.update_metadata::<RT>(update)
            }
            Msg::Ui(msg) => {
                let task = match &msg {
                    UiMsg::Selection(update) => self.update_state::<RT>(update.clone()),
                    UiMsg::Task(task) => self.exec_task::<RT>(task.clone()),
                    UiMsg::Custom(_) | UiMsg::Metadata(_) | UiMsg::RootDirUpdate(_) => Task::none(),
                };
                let ui = self.ui.update(msg).map(Msg::Ui);

                Task::batch([task, ui])
            }
        }
    }

    fn parse<RT: Runtime>(&self) -> Task<Msg<Ui>> {
        let metadata = Task::future(parse_metadata::<RT>(self.root_manifest()));
        let profiles = Task::future(parse_manual::<RT>(self.root_dir.clone()));
        Task::batch([metadata, profiles]).map(Msg::MetadataUpdate)
    }

    fn update_root_dir<RT: Runtime>(&mut self, root_dir: String) -> Task<Msg<Ui>> {
        let ui = Task::done(Msg::Ui(UiMsg::RootDirUpdate(root_dir.clone())));

        self.root_dir = root_dir;
        if let Some(s) = RT::get_state(self.state_key()) {
            self.state = s;
        }

        let parse = self.parse::<RT>();

        Task::batch([parse, ui])
    }

    fn update_state<RT: Runtime>(&mut self, update: selection::Update) -> Task<Msg<Ui>> {
        self.state.selection.update(update);
        Task::future(RT::persist_state(self.state_key(), self.state.clone())).discard()
    }

    fn exec_task<RT: Runtime>(&self, task: ui::Task) -> Task<Msg<Ui>> {
        match task {
            ui::Task::ImplicitCommand(implicit) => Task::done(Msg::Ui(UiMsg::Task(
                ui::Task::ExplicitCommand(implicit.to_explicit(&self.state.selection)),
            ))),
            ui::Task::ExplicitCommand(explicit) => {
                let (cmd, args, env) = {
                    let config = RT::get_configuration();
                    let ctx = explicit.task_context();

                    let config_cmd = config.get_cargo_command(ctx);
                    let mut cmd = config_cmd.split_whitespace().map(String::from);
                    let (cmd, mut args) = (cmd.next().unwrap(), cmd.collect::<Vec<_>>());
                    args.extend(explicit.into_args(&self.state.selection));
                    args.extend(config.get_extra_args(ctx));

                    (cmd, args, config.get_env(ctx))
                };
                Task::future(RT::exec_task(CargoTask::Cargo(runtime::Task {
                    cmd,
                    args,
                    env,
                })))
                .discard()
            }
            ui::Task::AddPlatformTarget(target) => {
                Task::future(RT::exec_task(CargoTask::Cargo(runtime::Task {
                    cmd: "rustup".to_string(),
                    args: vec!["target".to_string(), "add".to_string(), target],
                    env: HashMap::new(),
                })))
                .discard()
            }
        }
    }

    fn update_metadata<RT: Runtime>(&mut self, metadata_update: MetadataUpdate) -> Task<Msg<Ui>> {
        match &metadata_update {
            MetadataUpdate::Metadata(metadata) => {
                self.workspace_manifests = workspace_manifests(metadata);
            }
            MetadataUpdate::NoCargoToml => {
                self.workspace_manifests = Vec::new();
            }
            MetadataUpdate::FailedToRetrieve | MetadataUpdate::Profiles(_) => {}
        }

        let manifests = self
            .workspace_manifests
            .clone()
            .into_iter()
            .chain(iter::once(self.root_manifest()));

        let file_change = Task::future(async move {
            let notifiers = manifests.map(RT::file_changed_notifier);
            let ret = futures::stream::select_all(notifiers).next().await;
            RT::log("Manifest changed".to_string());
            ret
        })
        .and_then(|()| Task::done(Msg::ManifestUpdate));
        let ui = Task::done(UiMsg::Metadata(metadata_update)).map(Msg::Ui);

        Task::batch([file_change, ui])
    }

    pub fn subscription<RT: Runtime>(&self) -> Subscription<Msg<Ui>> {
        let root = Subscription::run(RT::current_dir_notitifier).map(Msg::RootDirUpdate);
        let ui = self.ui.subscription().map(Msg::Ui);

        Subscription::batch([root, ui])
    }

    pub fn state_key(&self) -> String {
        format!("{}.cargo_tools.cargo.state", self.root_dir)
    }

    pub fn root_manifest(&self) -> String {
        format!("{}/Cargo.toml", self.root_dir)
    }
}

trait TaskContext {
    fn task_context(&self) -> configuration::Context;
}

impl TaskContext for command::Explicit {
    fn task_context(&self) -> configuration::Context {
        match self {
            command::Explicit::Run(_) => configuration::Context::Run,
            command::Explicit::Test { package: _ } => configuration::Context::Test,
            command::Explicit::Build(_)
            | command::Explicit::Bench { package: _ }
            | command::Explicit::Doc
            | command::Explicit::Clean { package: _ } => configuration::Context::General,
        }
    }
}
