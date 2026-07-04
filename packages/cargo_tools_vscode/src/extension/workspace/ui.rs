use cargo_tools::cargo::{
    Profile,
    metadata::{
        Metadata, PackagesAndTargetDir, ParseError, parse_packages_and_target_dir, parse_profiles,
    },
};
use futures::channel::mpsc::channel;
use iced_viewless::Task;

use crate::{
    environment::metadata_task_context,
    extension::{
        send_file_changed,
        workspace::{configuration, outline},
    },
    runtime::{
        CHANNEL_CAPACITY, TsFileWatcher, exec_vs_code, file_exists_vs_code, read_file_vs_code,
        set_cargo_context,
    },
};
use tracing::error;

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    PackagesAndTargetDir(PackagesAndTargetDir),
    Profiles(Vec<Profile>),
    NoCargoToml,
    FailedToParse(String),
    CargoCommandEmpty(String),
}

impl MetadataUpdate {
    fn from_parse_manifest_result(res: Result<PackagesAndTargetDir, ParseError>) -> Self {
        match res {
            Ok(packages_and_target_dir) => Self::PackagesAndTargetDir(packages_and_target_dir),
            Err(err) => Self::from_parse_error(err),
        }
    }

    fn from_parse_profiles_result(res: Result<Vec<Profile>, ParseError>) -> Self {
        match res {
            Ok(profiles) => Self::Profiles(profiles),
            Err(err) => Self::from_parse_error(err),
        }
    }

    fn from_parse_error(err: ParseError) -> Self {
        match err {
            ParseError::NoCargoToml => Self::NoCargoToml,
            ParseError::Parse(e) => Self::FailedToParse(e.to_string()),
            ParseError::CargoCommandEmpty(e) => Self::CargoCommandEmpty(e.to_string()),
            ParseError::Exec(e) => Self::FailedToParse(e),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    ManifestChanged,
    ConfigFileChanged,
    MetadataChanged(MetadataUpdate),
    Configuration(configuration::Message),
    Outline(outline::Message),
}

pub struct Workspace {
    configuration: configuration::Configuration,
    outline: outline::Outline,
    metadata: Metadata,
    mainfests_file_watcher: TsFileWatcher,
    config_file_watcher: TsFileWatcher,
    root_dir: String,
}

impl Workspace {
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        // Init manifest file updates
        let (manifest_changed_tx, manifest_changed_rx) = channel(CHANNEL_CAPACITY);
        let manifests_file_watcher = TsFileWatcher::new(send_file_changed(manifest_changed_tx));

        // Init config file updates
        let (config_changed_tx, config_changed_rx) = channel(CHANNEL_CAPACITY);
        let config_file_watcher = TsFileWatcher::new(send_file_changed(config_changed_tx));

        let (configuration, configuration_task) =
            configuration::Configuration::init(root_dir.clone());
        let (outline, outline_task) = outline::Outline::init(root_dir.clone());

        let this = Self {
            configuration,
            outline,
            metadata: Metadata::default(),
            mainfests_file_watcher: manifests_file_watcher,
            config_file_watcher,
            root_dir,
        };

        this.config_file_watcher
            .watch_files(vec![this.root_manifest(), this.root_config()]);

        let task = Task::batch([
            // manifest and config updates will run for the lifetime of the extension
            Task::stream(manifest_changed_rx).map(|()| Message::ManifestChanged),
            Task::stream(config_changed_rx).map(|()| Message::ConfigFileChanged),
            // initially parse metadata
            this.parse_packages_and_target_dir(),
            this.parse_profiles(),
            // initial sub-component tasks
            configuration_task.map(Message::Configuration),
            outline_task.map(Message::Outline),
        ]);

        (this, task)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::MetadataChanged(update) => match update {
                MetadataUpdate::PackagesAndTargetDir(packages_and_target_dir) => {
                    self.metadata
                        .set_packages_and_target_dir(packages_and_target_dir);

                    // Update file watcher
                    let mut manifests = self.metadata.manifests();
                    manifests.push(self.root_manifest());
                    self.mainfests_file_watcher.watch_files(manifests);

                    let config = Task::done(Message::Configuration(
                        configuration::Message::ManifestFilesChanged,
                    ));
                    let outline = Task::done(Message::Outline(outline::Message::MetadataChanged));
                    let cargo_context = Task::future(set_cargo_context(true)).discard();

                    Task::batch([config, outline, cargo_context])
                }
                MetadataUpdate::Profiles(profiles) => {
                    self.metadata.set_profiles(profiles);
                    Task::none()
                }
                MetadataUpdate::NoCargoToml => {
                    // Always check for mainfest in root dir
                    self.mainfests_file_watcher
                        .watch_files(vec![self.root_manifest()]);

                    self.metadata = Metadata::default();

                    let config = Task::done(Message::Configuration(
                        configuration::Message::ManifestFilesChanged,
                    ));
                    let outline = Task::done(Message::Outline(outline::Message::MetadataChanged));
                    let cargo_context = Task::future(set_cargo_context(false)).discard();

                    Task::batch([config, outline, cargo_context])
                }
                // For invalid metadata or cargo command leave everything as is
                MetadataUpdate::CargoCommandEmpty(e) | MetadataUpdate::FailedToParse(e) => {
                    error!("{e}");
                    Task::none()
                }
            },
            Message::ManifestChanged => self.parse_packages_and_target_dir(),
            Message::ConfigFileChanged => self.parse_profiles(),
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

    fn parse_packages_and_target_dir(&self) -> Task<Message> {
        let root_manifest = self.root_manifest();
        Task::future(async move {
            if file_exists_vs_code(root_manifest.clone()).await {
                parse_packages_and_target_dir(root_manifest, metadata_task_context(), exec_vs_code)
                    .await
            } else {
                Err(ParseError::NoCargoToml)
            }
        })
        .map(MetadataUpdate::from_parse_manifest_result)
        .map(Message::MetadataChanged)
    }

    fn parse_profiles(&self) -> Task<Message> {
        let manifest = self.root_manifest();
        let config_toml = self.root_config();
        Task::future(async move {
            if file_exists_vs_code(manifest.clone()).await {
                Ok(parse_profiles(vec![manifest, config_toml], read_file_vs_code).await)
            } else {
                Err(ParseError::NoCargoToml)
            }
        })
        .map(MetadataUpdate::from_parse_profiles_result)
        .map(Message::MetadataChanged)
    }

    fn root_manifest(&self) -> String {
        format!("{}/Cargo.toml", self.root_dir)
    }

    fn root_config(&self) -> String {
        format!("{}/.cargo/config.toml", self.root_dir)
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
