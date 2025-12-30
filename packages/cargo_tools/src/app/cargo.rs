pub mod command;
pub mod metadata;
pub mod selection;
pub mod ui;

use std::iter;

use cargo_metadata::Metadata;
use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{
    app::cargo::metadata::{MetadataUpdate, parse_metadata, workspace_manifests},
    configuration::{self, Configuration},
    runtime::{self, CargoTask, Runtime},
};
pub enum CargoMessage {
    RootDirUpdate(String),
    ManifestUpdate,
    MetadataUpdate(MetadataUpdate),
    Ui(ui::Message),
}

use CargoMessage as Msg;

pub struct Cargo<Ui: ui::Ui> {
    root_dir: String,
    workspace_manifests: Vec<String>,
    metadata: Option<Metadata>,
    ui: Ui,
    state: ui::State,
}

impl<Ui: ui::Ui> Cargo<Ui> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT>(root_dir),
            Msg::ManifestUpdate => {
                Task::future(parse_metadata::<RT>(self.root_manifest())).map(Msg::MetadataUpdate)
            }
            Msg::MetadataUpdate(update) => self.update_metadata::<RT>(update),
            Msg::Ui(msg) => match msg {
                ui::Message::Update(update) => self.update_state::<RT>(update),
                ui::Message::Task(task) => self.exec_task::<RT>(task),
            },
        }
    }

    fn update_root_dir<RT: Runtime>(&mut self, root_dir: String) -> Task<Msg> {
        self.root_dir = root_dir;
        let selection = {
            if let Some(s) = RT::get_state(self.state_key()) {
                self.state = s;
            }
            Task::none()
        };

        let metadata =
            Task::future(parse_metadata::<RT>(self.root_dir.clone())).map(Msg::MetadataUpdate);

        Task::batch([metadata, selection])
    }

    fn update_state<RT: Runtime>(&mut self, update: ui::Update) -> Task<Msg> {
        if let Some(metadata) = &self.metadata {
            self.state.selection.update(update, metadata);
            Task::future(RT::persist_state(self.state_key(), self.state.clone())).discard()
        } else {
            Task::none()
        }
    }

    fn exec_task<RT: Runtime>(&self, task: ui::Task) -> Task<Msg> {
        match task {
            ui::Task::ImplicitCommand(implicit) => Task::done(Msg::Ui(ui::Message::Task(
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
        }
    }

    fn update_metadata<RT: Runtime>(&mut self, metadata_update: MetadataUpdate) -> Task<Msg> {
        self.ui.update(metadata_update.clone());

        match metadata_update {
            MetadataUpdate::New(metadata) => {
                self.workspace_manifests = workspace_manifests(&metadata);
                self.metadata = Some(metadata);
            }
            MetadataUpdate::NoCargoToml => {
                self.workspace_manifests = Vec::new();
                self.metadata = None;
            }
            MetadataUpdate::FailedToRetrieve => {}
        }

        let manifests = self
            .workspace_manifests
            .clone()
            .into_iter()
            .chain(iter::once(self.root_manifest()));

        Task::future(async move {
            let notifiers = manifests.map(RT::file_changed_notifier);
            futures::stream::select_all(notifiers).next().await;
        })
        .map(|()| Msg::ManifestUpdate)
    }

    pub fn subscription<RT: Runtime>(&self) -> Subscription<Msg> {
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
            | command::Explicit::Doc => configuration::Context::General,
        }
    }
}
