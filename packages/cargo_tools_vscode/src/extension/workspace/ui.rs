use cargo_tools::cargo::metadata::{Metadata, ParseError, parse_metadata};
use futures::channel::mpsc::channel;
use iced_viewless::Task;

use crate::{
    environment::metadata_task_context,
    extension::{
        send_file_changed,
        workspace::{configuration, outline},
    },
    runtime::{CHANNEL_CAPACITY, exec_vs_code, read_file_vs_code},
    vs_code_api::{TsFileWatcher, log_error, set_cargo_context},
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

#[derive(Debug)]
pub enum Message {
    ManifestChanged,
    MetadataChanged(MetadataUpdate),
    Configuration(configuration::Message),
    Outline(outline::Message),
}

pub struct Workspace {
    configuration: configuration::Configuration,
    outline: outline::Outline,
    metadata: Metadata,
    file_watcher: TsFileWatcher,
    root_dir: String,
}

impl Workspace {
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        // Init manifest file updates
        let (manifest_changed_tx, manifest_changed_rx) = channel(CHANNEL_CAPACITY);
        let file_watcher = TsFileWatcher::new(send_file_changed(manifest_changed_tx));

        let (configuration, configuration_task) =
            configuration::Configuration::init(root_dir.clone());
        let (outline, outline_task) = outline::Outline::init(root_dir.clone());

        let this = Self {
            configuration,
            outline,
            metadata: Metadata::default(),
            file_watcher,
            root_dir,
        };
        let task = Task::batch([
            // manifest updates will run for the lifetime of the extension
            Task::stream(manifest_changed_rx).map(|()| Message::ManifestChanged),
            // metadata is to initially parse the Cargo.toml
            this.parse_manifest(),
            configuration_task.map(Message::Configuration),
            outline_task.map(Message::Outline),
        ]);

        (this, task)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::MetadataChanged(update) => match update {
                MetadataUpdate::Metadata(metadata) => {
                    // Update file watcher
                    let mut manifests = metadata.manifests();
                    manifests.push(self.root_manifest());
                    self.file_watcher.watch_files(manifests);

                    self.metadata = metadata;

                    let config = Task::done(Message::Configuration(
                        configuration::Message::MetadataChanged,
                    ));
                    let outline = Task::done(Message::Outline(outline::Message::MetadataChanged));
                    let cargo_context = Task::future(set_cargo_context(true)).discard();

                    Task::batch([config, outline, cargo_context])
                }
                MetadataUpdate::NoCargoToml(e) => {
                    log_error(&e);
                    // Always check for mainfest in root dir
                    self.file_watcher.watch_files(vec![self.root_manifest()]);

                    self.metadata = Metadata::default();

                    let config = Task::done(Message::Configuration(
                        configuration::Message::MetadataChanged,
                    ));
                    let outline = Task::done(Message::Outline(outline::Message::MetadataChanged));
                    let cargo_context = Task::future(set_cargo_context(false)).discard();

                    Task::batch([config, outline, cargo_context])
                }
                // For invalid metadata or cargo command leave everything as is
                MetadataUpdate::CargoCommandEmpty(e) | MetadataUpdate::FailedToParse(e) => {
                    log_error(&e);
                    Task::none()
                }
            },
            Message::ManifestChanged => self.parse_manifest(),
            Message::Configuration(msg) => {
                let (task, event) = self.configuration.update(msg, &self.metadata);

                let task = task.map(Message::Configuration);
                match event {
                    Some(evt) => Task::batch([task, Task::done(evt.into_message())]),
                    None => task,
                }
            }
            Message::Outline(msg) => {
                let config = self.configuration.config();
                let (task, event) = self.outline.update(msg, &self.metadata, config);

                let task = task.map(Message::Outline);
                match event {
                    Some(evt) => Task::batch([task, Task::done(evt.into_message())]),
                    None => task,
                }
            }
        }
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

    fn root_manifest(&self) -> String {
        format!("{}/Cargo.toml", self.root_dir)
    }
}

trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for outline::Event {
    fn into_message(self) -> Message {
        match self {
            outline::Event::ConfigUpdate(update) => {
                Message::Configuration(configuration::Message::ConfigChanged(update))
            }
        }
    }
}

impl IntoMessage for configuration::Event {
    fn into_message(self) -> Message {
        match self {
            configuration::Event::ConfigUpdate => Message::Outline(outline::Message::ConfigChanged),
        }
    }
}
