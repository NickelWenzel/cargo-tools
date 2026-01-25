// mod project_outline;

use cargo_tools::app::cargo_make::ui::Task;
use iced_headless::Task as IcedTask;
use serde_wasm_bindgen::to_value;
use std::fmt::Debug;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    app::{
        CargoMakeMsg, SelectInput,
        cargo_make::{
            SettingsUpdate, Ui, UiMessage,
            command::{Command, Pinned},
        },
    },
    quick_pick::ToQuickPickItem,
    vs_code_api::{log, show_quick_pick, showInformationMessage},
};

trait IntoCargoMakeMessage {
    fn into_cargo_make_msg(self) -> CargoMakeMsg;
}

impl IntoCargoMakeMessage for Command {
    fn into_cargo_make_msg(self) -> CargoMakeMsg {
        CargoMakeMsg::Custom(UiMessage::Cmd(self))
    }
}

impl IntoCargoMakeMessage for SettingsUpdate {
    fn into_cargo_make_msg(self) -> CargoMakeMsg {
        CargoMakeMsg::Custom(UiMessage::Settings(self))
    }
}

async fn select<T: ToQuickPickItem + Clone + Debug + PartialEq>(
    SelectInput { options, current }: SelectInput<T>,
) -> Option<T> {
    let vccode_options = match options
        .iter()
        .map(|i| {
            let picked = current.contains(i);
            to_value(&i.to_item(picked))
        })
        .collect()
    {
        Ok(array) => array,
        Err(e) => {
            log(&format!("Failed to serialize quick pick items: {e:?}"));
            return None;
        }
    };

    let selected_index = match show_quick_pick(vccode_options).await {
        Ok(value) => value.as_f64().map(|f| f as usize),
        Err(e) => {
            log(&format!("Quick pick failed: {e:?}"));
            return None;
        }
    }?;

    options.get(selected_index).cloned()
}

impl Ui {
    pub(crate) fn process_cmd(&self, cmd: Command) -> IcedTask<CargoMakeMsg> {
        match cmd {
            Command::RunTask(task) => IcedTask::done(CargoMakeMsg::Task(Task::from_string(task))),
            Command::SelectAndRunTask => {
                let input = SelectInput {
                    options: self.makefile_tasks.clone(),
                    current: Vec::new(),
                };
                run_task(async move { select(input).await.map(|task| Command::RunTask(task.name)) })
            }
            Command::SelectTaskFilter => todo!(),
            Command::EditTaskFilter(filter) => {
                IcedTask::done(SettingsUpdate::TaskFilter(filter).into_cargo_make_msg())
            }
            Command::SelectCategoryFilter => todo!(),
            Command::EditCategoryFilter(filters) => {
                IcedTask::done(SettingsUpdate::CategoryFilter(filters).into_cargo_make_msg())
            }
            Command::ClearAllFilters => {
                let task = IcedTask::done(Command::EditTaskFilter(String::new()));
                let category = IcedTask::done(Command::EditCategoryFilter(Vec::new()));
                IcedTask::batch([task, category]).map(Command::into_cargo_make_msg)
            }
            Command::PinTask(task) => {
                IcedTask::done(SettingsUpdate::AddPinned(task).into_cargo_make_msg())
            }
            Command::Pinned(pinned) => match pinned {
                Pinned::Add => {
                    let input = SelectInput {
                        options: self.makefile_tasks.clone(),
                        current: Vec::new(),
                    };
                    run_task(async move { select(input).await.map(Command::PinTask) })
                }
                Pinned::Remove(idx) => {
                    IcedTask::done(SettingsUpdate::RemovePinned(idx).into_cargo_make_msg())
                }
                Pinned::Execute(task) => {
                    IcedTask::done(CargoMakeMsg::Task(Task::from_string(task)))
                }
                Pinned::Execute1 => self.execute_pinned(0),
                Pinned::Execute2 => self.execute_pinned(1),
                Pinned::Execute3 => self.execute_pinned(2),
                Pinned::Execute4 => self.execute_pinned(3),
                Pinned::Execute5 => self.execute_pinned(4),
            },
        }
    }

    fn execute_pinned(&self, idx: usize) -> IcedTask<CargoMakeMsg> {
        let Some(task) = self.settings.pinned_makefile_tasks.get(idx) else {
            showInformationMessage(
                format!("There is no task no. {} pinned ", idx + 1),
                Array::new(),
            );
            return IcedTask::none();
        };
        IcedTask::done(CargoMakeMsg::Task(Task::from_string(task.name.clone())))
    }
}

fn run_task(
    fut: impl Future<Output = Option<impl IntoCargoMakeMessage + 'static>> + 'static,
) -> IcedTask<CargoMakeMsg> {
    IcedTask::future(fut)
        .and_then(IcedTask::done)
        .map(IntoCargoMakeMessage::into_cargo_make_msg)
}
