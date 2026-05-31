use cargo_tools::xtask::{ParseError, PinnedAlias, XtaskAlias, XtaskAliases, parse_config};
use futures::{channel::mpsc::channel, future::join_all};
use iced_viewless::Task;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;

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
        TsFileWatcher, execute_task, set_xtask_context, show_input_box,
        show_quick_pick_with_buttons,
    },
};
use tracing::error;

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
    PendingEvent(Event),
}

#[derive(Debug, Clone)]
pub enum Event {
    TreeChanged(XtaskTreeProviderHandler),
    AddPinnedAlias(PinnedAlias),
}

#[derive(Debug)]
pub struct Xtask {
    aliases: XtaskAliases,
    settings: Settings,
    _cmds: Vec<CommandBinding>,
    _file_watcher: TsFileWatcher,
    root_dir: String,
}

impl Xtask {
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        let (config_changed_tx, config_changed_rx) = channel(CHANNEL_CAPACITY);
        let _file_watcher = TsFileWatcher::new(send_file_changed(config_changed_tx));
        _file_watcher.watch_files(vec![config_path(&root_dir)]);

        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_xtask_commands(cmd_tx);

        let settings: Settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let this = Self {
            aliases: XtaskAliases::default(),
            settings,
            _cmds,
            _file_watcher,
            root_dir,
        };

        let config_update = Task::stream(config_changed_rx).map(|()| Message::ConfigChanged);
        let cmd = Task::stream(cmd_rx).map(Message::Cmd);
        let initial_parse =
            Task::future(read_and_parse(config_path(&this.root_dir))).map(Message::AliasesChanged);

        (this, Task::batch([config_update, cmd, initial_parse]))
    }

    pub fn aliases(&self) -> &XtaskAliases {
        &self.aliases
    }

    pub fn name_filter(&self) -> &str {
        &self.settings.filter
    }

    pub fn update(&mut self, msg: Message) -> (Task<Message>, Option<Event>) {
        match msg {
            Message::AliasesChanged(result) => match result {
                Ok(aliases) => {
                    let has_aliases = !aliases.is_empty();
                    self.aliases = aliases;
                    let event = self.tree_changed_event();
                    (
                        Task::future(set_xtask_context(has_aliases)).discard(),
                        Some(event),
                    )
                }
                Err(e) => {
                    error!("{e}");
                    self.aliases = XtaskAliases::default();
                    let event = self.tree_changed_event();
                    (
                        Task::future(set_xtask_context(false)).discard(),
                        Some(event),
                    )
                }
            },
            Message::ConfigChanged => (
                Task::future(read_and_parse(config_path(&self.root_dir)))
                    .map(Message::AliasesChanged),
                None,
            ),
            Message::SettingsChanged(filter) => {
                self.settings.filter = filter;
                let event = self.tree_changed_event();
                let persist = Task::future(persist_state_vs_code(
                    settings_key(&self.root_dir),
                    self.settings.clone(),
                ))
                .discard();
                (persist, Some(event))
            }
            Message::Cmd(cmd) => self.handle_cmd(cmd),
            Message::PendingEvent(event) => (Task::none(), Some(event)),
        }
    }

    fn tree_changed_event(&self) -> Event {
        let aliases = self.aliases.filtered(&self.settings.filter);
        Event::TreeChanged(XtaskTreeProviderHandler::new(aliases))
    }

    fn handle_cmd(&self, cmd: Command) -> (Task<Message>, Option<Event>) {
        match cmd {
            Command::RunAlias(name) => (self.run_alias(name), None),
            Command::RunAliasWithArgs(name) => (
                Task::future(async move {
                    let placeholder = format!("Extra args for 'cargo {name}'");
                    let Ok(val) = show_input_box(placeholder, String::new()).await else {
                        return;
                    };
                    let Some(args_str) = val.as_string() else {
                        return;
                    };
                    let extra_args = args_str
                        .split_whitespace()
                        .map(String::from)
                        .collect::<Vec<_>>();
                    match XtaskAlias::try_into_process_with_extra_args(
                        name,
                        extra_args,
                        xtask_task_context(),
                    ) {
                        Ok(process) => execute_task(VsCodeTask::xtask_alias(process)).await,
                        Err(e) => error!("{e}"),
                    }
                })
                .discard(),
                None,
            ),
            Command::PinAlias(name) => (
                Task::none(),
                Some(Event::AddPinnedAlias(PinnedAlias {
                    name,
                    extra_args: vec![],
                })),
            ),
            Command::PinAliasWithArgs(name) => (
                Task::future(async move {
                    let placeholder =
                        format!("Args to always use when running 'cargo {name}' (pinned)");
                    let Ok(val) = show_input_box(placeholder, String::new()).await else {
                        return None;
                    };
                    val.as_string().map(|args_str| {
                        let extra_args = args_str.split_whitespace().map(String::from).collect();
                        Message::PendingEvent(Event::AddPinnedAlias(PinnedAlias {
                            name,
                            extra_args,
                        }))
                    })
                })
                .and_then(Task::done),
                None,
            ),
            Command::SelectAndRun => {
                let options = self.aliases.iter().cloned().collect::<Vec<_>>();
                let current = Vec::new();
                (
                    Task::future(async move {
                        SelectInput { options, current }
                            .select()
                            .await
                            .map(|alias| Command::RunAlias(alias.name))
                    })
                    .and_then(Task::done)
                    .map(Message::Cmd),
                    None,
                )
            }
            Command::SelectAndRunWithArgs => {
                let options = self.aliases.iter().cloned().collect::<Vec<_>>();
                (
                    Task::future(async move {
                        let tooltips = join_all(
                            options
                                .iter()
                                .map(|alias| super::tree_provider::fetch_tooltip(&alias.name)),
                        )
                        .await;
                        let items: Vec<QuickPickItem> = options
                            .iter()
                            .zip(tooltips)
                            .map(|(alias, help)| alias.to_item(false).with_button_tooltip(help))
                            .collect();
                        let vscode_options = match items.iter().map(to_value).collect() {
                            Ok(arr) => arr,
                            Err(e) => {
                                error!("Failed to serialize quick pick items: {e:?}");
                                return;
                            }
                        };
                        let selected_index =
                            match show_quick_pick_with_buttons(vscode_options).await {
                                Ok(v) => match v.as_f64().map(|f| f as usize) {
                                    Some(i) => i,
                                    None => return,
                                },
                                Err(e) => {
                                    error!("Quick pick failed: {e:?}");
                                    return;
                                }
                            };
                        let Some(alias) = options.get(selected_index).cloned() else {
                            return;
                        };
                        let placeholder = format!("Extra args for 'cargo {}'", alias.name);
                        let Ok(val) = show_input_box(placeholder, String::new()).await else {
                            return;
                        };
                        let Some(args_str) = val.as_string() else {
                            return;
                        };
                        let extra_args = args_str
                            .split_whitespace()
                            .map(String::from)
                            .collect::<Vec<_>>();
                        match XtaskAlias::try_into_process_with_extra_args(
                            alias.name,
                            extra_args,
                            xtask_task_context(),
                        ) {
                            Ok(process) => execute_task(VsCodeTask::xtask_alias(process)).await,
                            Err(e) => error!("{e}"),
                        }
                    })
                    .discard(),
                    None,
                )
            }
        }
    }

    fn run_alias(&self, name: String) -> Task<Message> {
        match XtaskAlias::try_into_process(name, xtask_task_context()) {
            Ok(process) => Task::future(execute_task(VsCodeTask::xtask_alias(process))).discard(),
            Err(e) => {
                error!("{e}");
                Task::none()
            }
        }
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
