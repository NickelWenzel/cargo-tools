use cargo_tools::xtask::{ParseError, XtaskAlias, XtaskAliases, parse_config};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_viewless::Task;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;

use crate::{
    environment::xtask_task_context,
    extension::{
        CommandBinding, send_file_changed,
        tasks::xtask::{
            command::{Command, register_xtask_commands},
            tree_provider::XtaskTreeProviderHandler,
        },
    },
    quick_pick::{QuickPickItem, SelectInput, ToQuickPickItem},
    runtime::{CHANNEL_CAPACITY, VsCodeTask, get_state_vs_code, persist_state_vs_code},
    vs_code_api::{
        TsFileWatcher, XtaskTreeProvider, execute_task, log_error, log_info, set_xtask_context,
        show_quick_pick_type,
    },
};

impl ToQuickPickItem for XtaskAlias {
    fn to_item(&self, _picked: bool) -> QuickPickItem {
        QuickPickItem::new(self.name.clone()).with_description(self.command_display())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ConfigChanged,
    AliasesChanged(Result<XtaskAliases, String>),
    SettingsChanged(String),
    Cmd(Command),
}

#[derive(Debug)]
pub struct Xtask {
    aliases: XtaskAliases,
    settings: Settings,
    tree_provider: XtaskTreeProvider,
    _cmds: Vec<CommandBinding>,
    _file_watcher: TsFileWatcher,
    root_dir: String,
    cmd_tx: Sender<Command>,
}

impl Xtask {
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        let (config_changed_tx, config_changed_rx) = channel(CHANNEL_CAPACITY);
        let _file_watcher = TsFileWatcher::new(send_file_changed(config_changed_tx));
        _file_watcher.watch_files(vec![config_path(&root_dir)]);

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_xtask_commands(cmd_tx.clone());

        let settings: Settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let handler = XtaskTreeProviderHandler::new(XtaskAliases::default());

        let this = Self {
            aliases: XtaskAliases::default(),
            settings,
            tree_provider: XtaskTreeProvider::new(handler),
            _cmds,
            _file_watcher,
            root_dir,
            cmd_tx,
        };

        let config_update = Task::stream(config_changed_rx).map(|()| Message::ConfigChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        let initial_parse =
            Task::future(read_and_parse(config_path(&this.root_dir))).map(Message::AliasesChanged);

        (this, Task::batch([config_update, cmd, initial_parse]))
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::AliasesChanged(result) => match result {
                Ok(aliases) => {
                    let has_aliases = !aliases.is_empty();
                    self.aliases = aliases;
                    self.update_ui();
                    Task::future(set_xtask_context(has_aliases)).discard()
                }
                Err(e) => {
                    log_error(&e);
                    self.aliases = XtaskAliases::default();
                    self.update_ui();
                    Task::future(set_xtask_context(false)).discard()
                }
            },
            Message::ConfigChanged => Task::future(read_and_parse(config_path(&self.root_dir)))
                .map(Message::AliasesChanged),
            Message::SettingsChanged(filter) => {
                self.settings.filter = filter;
                self.update_ui();
                Task::future(persist_state_vs_code(
                    settings_key(&self.root_dir),
                    self.settings.clone(),
                ))
                .discard()
            }
            Message::Cmd(cmd) => self.handle_cmd(cmd),
        }
    }

    fn update_ui(&self) {
        let aliases = self.aliases.filtered(&self.settings.filter);
        self.tree_provider
            .update(XtaskTreeProviderHandler::new(aliases));
    }

    fn handle_cmd(&self, cmd: Command) -> Task<Message> {
        match cmd {
            Command::RunAlias(name) => self.run_alias(name),
            Command::SelectAndRun => {
                let options = self.aliases.iter().cloned().collect::<Vec<_>>();
                let current = Vec::new();
                Task::future(async move {
                    SelectInput { options, current }
                        .select()
                        .await
                        .map(|alias| Command::RunAlias(alias.name))
                })
                .and_then(Task::done)
                .map(Message::Cmd)
            }
            Command::SelectFilter => self.select_filter(),
            Command::EditFilter(filter) => Task::done(Message::SettingsChanged(filter)),
            Command::ClearFilter => Task::done(Message::SettingsChanged(String::new())),
        }
    }

    fn run_alias(&self, name: String) -> Task<Message> {
        match XtaskAlias::try_into_process(name, xtask_task_context()) {
            Ok(process) => Task::future(execute_task(VsCodeTask::xtask_alias(process))).discard(),
            Err(e) => {
                log_error(&e.to_string());
                Task::none()
            }
        }
    }

    fn select_filter(&self) -> Task<Message> {
        let current = self.settings.filter.clone();
        let Ok(options) = self
            .aliases
            .iter()
            .map(|a| to_value(&a.to_item(false)))
            .collect()
        else {
            return Task::none();
        };

        let cmd_tx = self.cmd_tx.clone();

        Task::future(async move {
            let filter_update = Closure::new(move |filter: String| {
                let mut tx = cmd_tx.clone();
                spawn_local(async move {
                    log_info(&format!("Sending xtask filter '{filter}'"));
                    if let Err(e) = tx.send(Command::EditFilter(filter)).await {
                        log_error(&format!("Failed to queue msg: {e}"));
                    }
                });
            });

            let filter = show_quick_pick_type(current.clone(), options, &filter_update)
                .await
                .map(|f| f.as_string().unwrap_or(current.clone()))
                .unwrap_or(current);

            Command::EditFilter(filter)
        })
        .map(Message::Cmd)
    }
}

async fn read_and_parse(config: String) -> Result<XtaskAliases, String> {
    let content = crate::runtime::read_file_vs_code(config)
        .await
        .map_err(|e| format!("Failed to read .cargo/config.toml: {e}"))?;
    parse_config(&content).map_err(|e: ParseError| e.to_string())
}

fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.tasks.xtask.ui_settings")
}

fn config_path(root_dir: &str) -> String {
    format!("{root_dir}/.cargo/config.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    filter: String,
}
