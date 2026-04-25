use std::ops::Deref;

use cargo_tools::cargo_make::MakefileTask;
use futures::SinkExt;
use iced_headless::Task;
use itertools::Itertools;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;

use crate::{
    environment::{TaskContext, environment},
    extension::cargo_make::{
        Message, SettingsUpdate, Ui,
        command::{Command, Pinned},
    },
    quick_pick::{SelectInput, ToQuickPickItem},
    runtime::exec_task_vs_code,
    vs_code_api::{log, show_quick_pick_type, showInformationMessage},
};

trait IntoCargoMakeMessage {
    fn into_cargo_make_msg(self) -> Message;
}

impl IntoCargoMakeMessage for Command {
    fn into_cargo_make_msg(self) -> Message {
        Message::Cmd(self)
    }
}

impl IntoCargoMakeMessage for SettingsUpdate {
    fn into_cargo_make_msg(self) -> Message {
        Message::SettingsChanged(self)
    }
}

impl Ui {
    pub(crate) fn process_cmd(&self, cmd: Command) -> Task<Message> {
        match cmd {
            Command::RunTask(task) => self.make_task_exec(task),
            Command::SelectAndRunTask => {
                let input = SelectInput {
                    options: self.makefile_tasks.deref().clone(),
                    current: Vec::new(),
                };
                done(async move { input.select().await.map(|task| Command::RunTask(task.name)) })
            }
            Command::SelectTaskFilter => self.select_task_filter(),
            Command::EditTaskFilter(filter) => {
                Task::done(SettingsUpdate::TaskFilter(filter).into_cargo_make_msg())
            }
            Command::SelectCategoryFilter => self.select_category_filter(),
            Command::EditCategoryFilter(category_filters) => {
                Task::done(SettingsUpdate::CategoryFilter(category_filters).into_cargo_make_msg())
            }
            Command::ClearAllFilters => {
                let task = Task::done(SettingsUpdate::TaskFilter(String::new()));
                let category = Task::done(SettingsUpdate::CategoryFilter(Vec::new()));
                Task::batch([task, category]).map(SettingsUpdate::into_cargo_make_msg)
            }
            Command::PinTask(task) => self
                .makefile_tasks
                .iter()
                .find(|t| t.name == task)
                .cloned()
                .map(|task| Task::done(SettingsUpdate::AddPinned(task).into_cargo_make_msg()))
                .unwrap_or(Task::none()),
            Command::Pinned(pinned) => match pinned {
                Pinned::Add => {
                    let input = SelectInput {
                        options: self.makefile_tasks.deref().clone(),
                        current: Vec::new(),
                    };
                    done(async move { input.select().await.map(|t| Command::PinTask(t.name)) })
                }
                Pinned::Remove(task) => {
                    let Some(idx) = self
                        .settings
                        .pinned_makefile_tasks
                        .iter()
                        .position(|pinned| pinned.name == task)
                    else {
                        return Task::none();
                    };
                    Task::done(SettingsUpdate::RemovePinned(idx).into_cargo_make_msg())
                }
                Pinned::Execute(task) => self.make_task_exec(task),
                Pinned::Execute1 => self.execute_pinned(0),
                Pinned::Execute2 => self.execute_pinned(1),
                Pinned::Execute3 => self.execute_pinned(2),
                Pinned::Execute4 => self.execute_pinned(3),
                Pinned::Execute5 => self.execute_pinned(4),
            },
        }
    }

    fn execute_pinned(&self, idx: usize) -> Task<Message> {
        match self.settings.pinned_makefile_tasks.get(idx) {
            Some(task) => self.make_task_exec(task.name.clone()),
            None => Task::future(showInformationMessage(
                format!("There is no task no. {} pinned ", idx + 1),
                Vec::new(),
            ))
            .discard(),
        }
    }

    fn make_task_exec(&self, make_task: String) -> Task<Message> {
        let task = MakefileTask::into_task(make_task, environment(TaskContext::General));
        Task::future(exec_task_vs_code(task)).discard()
    }

    fn select_task_filter(&self) -> Task<Message> {
        let current = self.settings.task_filter.clone();
        let Ok(options) = self
            .makefile_tasks
            .iter()
            .map(|i| to_value(&i.to_item(false)))
            .collect()
        else {
            return Task::none();
        };

        let cmd_tx = self.cmd_tx.clone();

        Task::future(async move {
            // Closure only needs to live while the quickpick is active
            let filter_update = Closure::new(move |filter: String| {
                let mut tx = cmd_tx.clone();
                spawn_local(async move {
                    log(&format!("Sending cargo make task filter '{filter}'"));
                    if let Err(e) = tx.send(Command::EditTaskFilter(filter)).await {
                        log(&format!("Failed to queue msg: {}", e));
                    }
                });
            });

            let filter = show_quick_pick_type(current.clone(), options, &filter_update)
                .await
                .map(|f| f.as_string().unwrap_or(current.clone()))
                .unwrap_or(current);

            Command::EditTaskFilter(filter)
        })
        .map(Message::Cmd)
    }

    fn select_category_filter(&self) -> Task<Message> {
        let categories = self
            .makefile_tasks
            .iter()
            .map(|task| &task.category)
            .unique();

        let current = self.settings.category_filters.clone();
        // Select all categories that are not filtered out
        let selected = categories
            .clone()
            .filter(|category| !current.contains(category))
            .cloned()
            .collect();

        let categories: Vec<_> = categories.cloned().collect();
        let input = SelectInput {
            options: categories.clone(),
            current: selected,
        };

        let cmd_tx = self.cmd_tx.clone();
        let categories_filter_update = categories.clone();
        let filter_update = move |selected: Vec<String>| {
            log(&format!(
                "Received category filter update from quickpick'{selected:?}'"
            ));
            let mut tx = cmd_tx.clone();
            let categories = categories_filter_update.clone();
            spawn_local(async move {
                let selected = categories
                    .iter()
                    .filter(|category| !selected.contains(category))
                    .cloned()
                    .collect();
                log(&format!(
                    "Sending cargo make category filter '{selected:?}'"
                ));
                if let Err(e) = tx.send(Command::EditCategoryFilter(selected)).await {
                    log(&format!("Failed to queue msg: {}", e));
                }
            });
        };

        Task::future(async move {
            let selected_categories = input
                .select_multiple(filter_update)
                .await
                .map(|selected| {
                    // Select all categories that are not filtered out
                    categories
                        .into_iter()
                        .filter(|category| !selected.contains(category))
                        .collect()
                })
                .unwrap_or(current);

            SettingsUpdate::CategoryFilter(selected_categories)
        })
        .map(Message::SettingsChanged)
    }
}

fn done(
    fut: impl Future<Output = Option<impl IntoCargoMakeMessage + 'static>> + 'static,
) -> Task<Message> {
    Task::future(fut)
        .and_then(Task::done)
        .map(IntoCargoMakeMessage::into_cargo_make_msg)
}
