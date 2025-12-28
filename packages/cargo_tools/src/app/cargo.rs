pub mod command;
pub mod metadata;
pub mod selection;
pub mod ui;

use std::{collections::HashMap, iter};

use cargo_metadata::Metadata;
use futures::StreamExt;
use iced_headless::{Subscription, Task};

use crate::{
    app::cargo::metadata::{parse_metadata, workspace_manifests, MetadataUpdate},
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
    root_manifest: String,
    workspace_manifests: Vec<String>,
    metadata: Option<Metadata>,
    ui: Ui,
    state: ui::State,
}

impl<Ui: ui::Ui> Cargo<Ui> {
    pub fn update<RT: Runtime>(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::RootDirUpdate(root_dir) => self.update_root_dir::<RT>(root_dir),
            Msg::ManifestUpdate => Task::future(parse_metadata::<RT>(self.root_manifest.clone()))
                .map(Msg::MetadataUpdate),
            Msg::MetadataUpdate(update) => self.update_metadata::<RT>(update),
            Msg::Ui(msg) => match msg {
                ui::Message::Update(update) => self.update_state(update),
                ui::Message::Task(task) => self.exec_task::<RT>(task),
            },
        }
    }

    fn update_root_dir<RT: Runtime>(&mut self, root_dir: String) -> Task<Msg> {
        self.root_manifest = format!("{root_dir}/Cargo.toml");
        let selection = {
            if let Some(s) = RT::get_state(format!("{root_dir}.cargo_selection")) {
                self.state.selection = s;
            }
            Task::none()
        };

        let metadata =
            Task::future(parse_metadata::<RT>(self.root_manifest.clone())).map(Msg::MetadataUpdate);

        Task::batch([metadata, selection])
    }

    fn update_state(&mut self, update: ui::Update) -> Task<Msg> {
        if let Some(metadata) = &self.metadata {
            self.state.selection.update(update, metadata);
        }
        Task::none()
    }

    fn exec_task<RT: Runtime>(&self, task: ui::Task) -> Task<Msg> {
        match task {
            ui::Task::ImplicitCommand(implicit) => Task::done(Msg::Ui(ui::Message::Task(
                ui::Task::ExplicitCommand(implicit.to_explicit(&self.state.selection)),
            ))),
            ui::Task::ExplicitCommand(explicit) => {
                let args = explicit.into_args(&self.state.selection);
                Task::future(RT::exec_task(CargoTask::Cargo(runtime::Task {
                    cmd: "cargo".to_string(),
                    args,
                    env: HashMap::new(),
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
            .chain(iter::once(self.root_manifest.clone()));

        Task::future(async move {
            let notifiers = manifests.map(RT::file_changed_notifier);
            futures::stream::select_all(notifiers).next().await;
        })
        .map(|()| Msg::ManifestUpdate)
    }

    pub fn subscription<RuntimeT: Runtime>(&self) -> Subscription<Msg> {
        Subscription::run(RuntimeT::current_dir_notitifier).map(Msg::RootDirUpdate)
    }
}
