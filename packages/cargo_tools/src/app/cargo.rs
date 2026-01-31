pub mod command;
pub mod metadata;
pub mod selection;
pub mod ui;

use std::iter;

use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{
    app::cargo::metadata::{MetadataUpdate, parse_metadata, parse_profiles, workspace_manifests},
    configuration::{self},
    runtime::Runtime,
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

#[derive(Debug, Default)]
pub struct Cargo<Ui: ui::Ui + Default> {
    root_dir: String,
    workspace_manifests: Vec<String>,
    ui: Ui,
    state: ui::State,
}

impl<Ui: ui::Ui + Default + 'static> Cargo<Ui> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg<Ui>) -> Task<Msg<Ui>> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT>(root_dir),
            Msg::ManifestUpdate => self.parse::<RT>(),
            Msg::ConfigUpdate => self.parse::<RT>(),
            Msg::MetadataUpdate(update) => self.update_metadata::<RT>(update),
            Msg::Ui(msg) => {
                let task = match &msg {
                    UiMsg::Selection(update) => self.update_state::<RT>(update.clone()),
                    UiMsg::Custom(_) | UiMsg::Metadata(_) | UiMsg::RootDirUpdate(_) => Task::none(),
                };
                let ui = self.ui.update(msg).map(Msg::Ui);

                Task::batch([task, ui])
            }
        }
    }

    fn parse<RT: Runtime>(&self) -> Task<Msg<Ui>> {
        let metadata = Task::future(parse_metadata::<RT>(self.root_manifest()));
        let profiles = Task::future(parse_profiles::<RT>(self.root_dir.clone()));
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
