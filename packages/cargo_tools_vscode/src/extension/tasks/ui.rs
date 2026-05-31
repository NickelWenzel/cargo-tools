use std::iter;

use cargo_tools::xtask::XtaskAliases;
use futures::channel::mpsc::{Sender, channel};
use iced_viewless::Task;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    commands::tasks::*,
    extension::{
        select_name_filter,
        tasks::{
            cargo_make::{self, tree_provider::CargoMakeTreeProviderHandler},
            pinned::{self, SettingsUpdate},
            xtask::{self, tree_provider::XtaskTreeProviderHandler},
        },
        vscode_task_utils::{CommandBinding, register_commands},
    },
    quick_pick::ToQuickPickItem,
    runtime::CHANNEL_CAPACITY,
    vs_code_api::TasksTreeProvider,
};

use cargo_tools::cargo_make::MakefileTasks;

#[derive(Debug, Clone)]
pub enum SharedCommand {
    SelectNameFilter,
    EditNameFilter(String),
    ClearAllFilters,
}

#[derive(Debug)]
pub enum Message {
    CargoMake(cargo_make::Message),
    Pinned(pinned::Message),
    Xtask(xtask::Message),
    UpdateCargoMakeTree(CargoMakeTreeProviderHandler),
    UpdateXtaskTree(XtaskTreeProviderHandler),
    SharedCmd(SharedCommand),
}

pub struct Tasks {
    cargo_make: cargo_make::CargoMake,
    pinned: pinned::Pinned,
    xtask: xtask::Xtask,
    tasks_tree: TasksTreeProvider,
    shared_cmd_tx: Sender<SharedCommand>,
    _shared_cmds: Vec<CommandBinding>,
}

impl Tasks {
    pub fn init(root_dir: String) -> (Self, Task<Message>) {
        let (cargo_make, cargo_make_task) = cargo_make::CargoMake::init(root_dir.clone());
        let (pinned, pinned_task) = pinned::Pinned::init(root_dir.clone());
        let (xtask, xtask_task) = xtask::Xtask::init(root_dir);

        let initial_cm_handler = CargoMakeTreeProviderHandler::new(MakefileTasks::default());
        let initial_xt_handler = XtaskTreeProviderHandler::new(XtaskAliases::default());
        let tasks_tree = TasksTreeProvider::new(initial_cm_handler, initial_xt_handler);

        let (shared_cmd_tx, shared_cmd_rx) = channel(CHANNEL_CAPACITY);
        let _shared_cmds = register_commands(
            shared_cmd_tx.clone(),
            [
                (CARGO_TOOLS_TASKS_SELECT_NAME_FILTER, |_| {
                    Some(SharedCommand::SelectNameFilter)
                }),
                (CARGO_TOOLS_TASKS_CLEAR_ALL_FILTERS, |_| {
                    Some(SharedCommand::ClearAllFilters)
                }),
            ],
        );

        let this = Self {
            cargo_make,
            pinned,
            xtask,
            tasks_tree,
            shared_cmd_tx,
            _shared_cmds,
        };
        let task = Task::batch([
            cargo_make_task.map(Message::CargoMake),
            pinned_task.map(Message::Pinned),
            xtask_task.map(Message::Xtask),
            Task::stream(shared_cmd_rx).map(Message::SharedCmd),
        ]);

        (this, task)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::CargoMake(msg) => {
                let (task, event) = self.cargo_make.update(msg);
                Task::batch(
                    iter::once(task.map(Message::CargoMake))
                        .chain(event.map(|evt| Task::done(evt.into_message()))),
                )
            }
            Message::Pinned(msg) => self
                .pinned
                .update(self.cargo_make.makefile_tasks(), self.xtask.aliases(), msg)
                .map(Message::Pinned),
            Message::Xtask(msg) => {
                let (task, event) = self.xtask.update(msg);
                Task::batch(
                    iter::once(task.map(Message::Xtask))
                        .chain(event.map(|evt| Task::done(evt.into_message()))),
                )
            }
            Message::UpdateCargoMakeTree(handler) => {
                self.tasks_tree.update_cargo_make(handler);
                Task::none()
            }
            Message::UpdateXtaskTree(handler) => {
                self.tasks_tree.update_xtask(handler);
                Task::none()
            }
            Message::SharedCmd(cmd) => self.handle_shared_cmd(cmd),
        }
    }

    fn handle_shared_cmd(&self, cmd: SharedCommand) -> Task<Message> {
        match cmd {
            SharedCommand::SelectNameFilter => {
                let cm = self.cargo_make.name_filter();
                // If the two panels have diverged (only possible from legacy persisted state
                // before the shared filter existed), start fresh rather than silently
                // promoting one panel's stale value as canonical.
                let current = if cm == self.xtask.name_filter() {
                    cm.to_owned()
                } else {
                    String::new()
                };
                let options = Array::new();
                for item in self.cargo_make.makefile_tasks().iter() {
                    if let Ok(v) = to_value(&item.to_item(false)) {
                        options.push(&v);
                    }
                }
                for item in self.xtask.aliases().iter() {
                    if let Ok(v) = to_value(&item.to_item(false)) {
                        options.push(&v);
                    }
                }
                select_name_filter(
                    current,
                    options,
                    self.shared_cmd_tx.clone(),
                    SharedCommand::EditNameFilter,
                )
                .map(Message::SharedCmd)
            }
            SharedCommand::EditNameFilter(filter) => Task::batch([
                Task::done(Message::CargoMake(cargo_make::Message::SettingsChanged(
                    cargo_make::SettingsUpdate::TaskFilter(filter.clone()),
                ))),
                Task::done(Message::Xtask(xtask::Message::SettingsChanged(filter))),
            ]),
            SharedCommand::ClearAllFilters => Task::batch([
                Task::done(Message::CargoMake(cargo_make::Message::SettingsChanged(
                    cargo_make::SettingsUpdate::TaskFilter(String::new()),
                ))),
                Task::done(Message::CargoMake(cargo_make::Message::SettingsChanged(
                    cargo_make::SettingsUpdate::CategoryFilter(Vec::new()),
                ))),
                Task::done(Message::Xtask(xtask::Message::SettingsChanged(
                    String::new(),
                ))),
            ]),
        }
    }
}

trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for cargo_make::Event {
    fn into_message(self) -> Message {
        match self {
            cargo_make::Event::AddPinned(task) => Message::Pinned(
                pinned::Message::SettingsChanged(SettingsUpdate::AddPinned(task)),
            ),
            cargo_make::Event::TreeChanged(handler) => Message::UpdateCargoMakeTree(handler),
        }
    }
}

impl IntoMessage for xtask::Event {
    fn into_message(self) -> Message {
        match self {
            xtask::Event::TreeChanged(handler) => Message::UpdateXtaskTree(handler),
            xtask::Event::AddPinnedAlias(alias) => Message::Pinned(
                pinned::Message::SettingsChanged(SettingsUpdate::AddPinnedAlias(alias)),
            ),
        }
    }
}
