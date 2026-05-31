use std::iter;

use cargo_tools::{
    cargo_make::{MakefileTask, MakefileTasks},
    xtask::{PinnedAlias, XtaskAlias, XtaskAliases},
};
use futures::channel::mpsc::channel;
use iced_viewless::Task;

use serde::{Deserialize, Serialize};

use crate::{
    environment::{makefile_task_context, xtask_task_context},
    extension::{
        CommandBinding,
        tasks::pinned::{
            command::{Command, register_pinned_commands},
            tree_provider::CargoMakePinnedTreeProviderHandler,
        },
    },
    quick_pick::{QuickPickItem, SelectInput, ToQuickPickItem},
    runtime::{CHANNEL_CAPACITY, VsCodeTask, get_state_vs_code, persist_state_vs_code},
    vs_code_api::{
        CargoMakePinnedTreeProvider, execute_task, log_error, show_input_box,
        showInformationMessage,
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    SettingsChanged(SettingsUpdate),
    Cmd(Command),
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    AddPinned(MakefileTask),
    RemovePinned(usize),
    AddPinnedAlias(PinnedAlias),
    RemovePinnedAlias(usize),
}

#[derive(Debug)]
pub struct Pinned {
    settings: Settings,
    ui: CargoMakePinnedTreeProvider,
    _cmds: Vec<CommandBinding>,
    root_dir: String,
}

impl Pinned {
    /// Inits all data and update channels
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        let (cmd_tx, cmd_rx) = channel(CHANNEL_CAPACITY);
        let _cmds = register_pinned_commands(cmd_tx);

        let settings: Settings = get_state_vs_code(settings_key(&root_dir)).unwrap_or_default();

        let handler = CargoMakePinnedTreeProviderHandler::new(
            settings.pinned_makefile_tasks.clone(),
            settings.pinned_aliases.clone(),
        );

        let this = Self {
            settings,
            ui: CargoMakePinnedTreeProvider::new(handler),
            _cmds,
            root_dir,
        };

        let cmd = Task::stream(cmd_rx).map(Message::Cmd);

        (this, cmd)
    }

    pub fn update(
        &mut self,
        makefile_tasks: &MakefileTasks,
        xtask_aliases: &XtaskAliases,
        msg: Message,
    ) -> Task<Message> {
        match msg {
            Message::SettingsChanged(update) => self.update_state(update),
            Message::Cmd(cmd) => self.handle_cmd(makefile_tasks, xtask_aliases, cmd),
        }
    }

    fn update_state(&mut self, update: SettingsUpdate) -> Task<Message> {
        let Settings {
            pinned_makefile_tasks,
            pinned_aliases,
        } = &mut self.settings;

        let msg_task = match update {
            SettingsUpdate::AddPinned(task) => {
                if !pinned_makefile_tasks.contains(&task) {
                    pinned_makefile_tasks.push(task);
                    self.update_ui();
                    None
                } else {
                    Some(
                        Task::future(showInformationMessage(
                            format!("Task '{}' is already pinned.", task.name),
                            Vec::new(),
                        ))
                        .discard(),
                    )
                }
            }
            SettingsUpdate::RemovePinned(idx) => {
                if idx < pinned_makefile_tasks.len() {
                    pinned_makefile_tasks.remove(idx);
                    self.update_ui();
                    None
                } else {
                    Some(
                        Task::future(showInformationMessage(
                            format!("Task no. '{idx}' could not be unpinned."),
                            Vec::new(),
                        ))
                        .discard(),
                    )
                }
            }
            SettingsUpdate::AddPinnedAlias(alias) => {
                if !pinned_aliases.contains(&alias) {
                    pinned_aliases.push(alias);
                    self.update_ui();
                    None
                } else {
                    Some(
                        Task::future(showInformationMessage(
                            format!("Alias '{}' with these args is already pinned.", alias.name),
                            Vec::new(),
                        ))
                        .discard(),
                    )
                }
            }
            SettingsUpdate::RemovePinnedAlias(idx) => {
                if idx < pinned_aliases.len() {
                    pinned_aliases.remove(idx);
                    self.update_ui();
                    None
                } else {
                    Some(
                        Task::future(showInformationMessage(
                            format!("Alias no. '{idx}' could not be unpinned."),
                            Vec::new(),
                        ))
                        .discard(),
                    )
                }
            }
        };

        let tasks = iter::once(
            Task::future(persist_state_vs_code(
                settings_key(&self.root_dir),
                self.settings.clone(),
            ))
            .discard(),
        )
        .chain(msg_task);

        Task::batch(tasks)
    }

    fn update_ui(&self) {
        let handler = CargoMakePinnedTreeProviderHandler::new(
            self.settings.pinned_makefile_tasks.clone(),
            self.settings.pinned_aliases.clone(),
        );
        self.ui.update(handler);
    }

    fn handle_cmd(
        &self,
        makefile_tasks: &MakefileTasks,
        xtask_aliases: &XtaskAliases,
        cmd: Command,
    ) -> Task<Message> {
        match cmd {
            Command::Add => {
                let options: Vec<PinnableItem> = makefile_tasks
                    .iter()
                    .cloned()
                    .map(PinnableItem::Task)
                    .chain(xtask_aliases.iter().cloned().map(PinnableItem::Alias))
                    .collect();
                let input = SelectInput {
                    options,
                    current: Vec::new(),
                };
                done(async move {
                    let selected = input.select().await?;
                    match selected {
                        PinnableItem::Task(task) => Some(SettingsUpdate::AddPinned(task)),
                        PinnableItem::Alias(alias) => {
                            let placeholder = format!(
                                "Args to always use when running 'cargo {}' (pinned)",
                                alias.name
                            );
                            let Ok(val) = show_input_box(placeholder, String::new()).await else {
                                return None;
                            };
                            let args_str = val.as_string().unwrap_or_default();
                            let extra_args =
                                args_str.split_whitespace().map(String::from).collect();
                            Some(SettingsUpdate::AddPinnedAlias(PinnedAlias {
                                name: alias.name,
                                extra_args,
                            }))
                        }
                    }
                })
            }
            Command::Remove(task) => {
                let Some(idx) = self
                    .settings
                    .pinned_makefile_tasks
                    .iter()
                    .position(|pinned| pinned.name == task)
                else {
                    return Task::none();
                };
                Task::done(SettingsUpdate::RemovePinned(idx).into_pinned_msg())
            }
            Command::Execute(task) => self.make_task_exec(task),
            Command::ExecuteAlias(key) => self.alias_exec_by_key(key),
            Command::RemoveAlias(key) => self.alias_remove_by_key(key),
            Command::Execute1 => self.execute_pinned(0),
            Command::Execute2 => self.execute_pinned(1),
            Command::Execute3 => self.execute_pinned(2),
            Command::Execute4 => self.execute_pinned(3),
            Command::Execute5 => self.execute_pinned(4),
        }
    }

    /// Execute the Nth item across the combined list (makefile tasks first, then aliases).
    fn execute_pinned(&self, idx: usize) -> Task<Message> {
        let task_count = self.settings.pinned_makefile_tasks.len();
        if idx < task_count {
            match self.settings.pinned_makefile_tasks.get(idx) {
                Some(task) => self.make_task_exec(task.name.clone()),
                None => Task::none(),
            }
        } else {
            let alias_idx = idx - task_count;
            match self.settings.pinned_aliases.get(alias_idx) {
                Some(alias) => self.alias_exec(alias.clone()),
                None => Task::future(showInformationMessage(
                    format!("There is no task no. {} pinned ", idx + 1),
                    Vec::new(),
                ))
                .discard(),
            }
        }
    }

    fn make_task_exec(&self, make_task: String) -> Task<Message> {
        match MakefileTask::try_into_process(make_task, makefile_task_context()) {
            Ok(process) => Task::future(execute_task(VsCodeTask::cargo_make(process))).discard(),
            Err(e) => {
                log_error(&e.to_string());
                Task::none()
            }
        }
    }

    fn alias_exec_by_key(&self, key: String) -> Task<Message> {
        self.alias_exec(pinned_alias_from_key(&key))
    }

    fn alias_remove_by_key(&self, key: String) -> Task<Message> {
        let target = pinned_alias_from_key(&key);
        let Some(idx) = self
            .settings
            .pinned_aliases
            .iter()
            .position(|a| a == &target)
        else {
            return Task::none();
        };
        Task::done(SettingsUpdate::RemovePinnedAlias(idx).into_pinned_msg())
    }

    fn alias_exec(&self, alias: PinnedAlias) -> Task<Message> {
        let result = XtaskAlias::try_into_process_with_extra_args(
            alias.name,
            alias.extra_args,
            xtask_task_context(),
        );
        match result {
            Ok(process) => Task::future(execute_task(VsCodeTask::xtask_alias(process))).discard(),
            Err(e) => {
                log_error(&e.to_string());
                Task::none()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum PinnableItem {
    Task(MakefileTask),
    Alias(XtaskAlias),
}

impl ToQuickPickItem for PinnableItem {
    fn to_item(&self, picked: bool) -> QuickPickItem {
        match self {
            Self::Task(t) => t.to_item(picked),
            Self::Alias(a) => a.to_item(picked),
        }
    }
}

trait IntoPinnedMessage {
    fn into_pinned_msg(self) -> Message;
}

impl IntoPinnedMessage for SettingsUpdate {
    fn into_pinned_msg(self) -> Message {
        Message::SettingsChanged(self)
    }
}

fn done(
    fut: impl Future<Output = Option<impl IntoPinnedMessage + 'static>> + 'static,
) -> Task<Message> {
    Task::future(fut)
        .and_then(Task::done)
        .map(IntoPinnedMessage::into_pinned_msg)
}

/// Decodes a `"name|args_string"` composite key back into a `PinnedAlias`.
fn pinned_alias_from_key(key: &str) -> PinnedAlias {
    let (name, args_str) = key.split_once('|').unwrap_or((key, ""));
    let extra_args = args_str.split_whitespace().map(String::from).collect();
    PinnedAlias {
        name: name.to_string(),
        extra_args,
    }
}

fn settings_key(root_dir: &str) -> String {
    format!("{root_dir}.cargo_tools.tasks.pinned.ui_settings")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pinned_makefile_tasks: MakefileTasks,
    #[serde(default)]
    pinned_aliases: Vec<PinnedAlias>,
}
