// mod project_outline;

use cargo_tools::app::cargo_make::ui::Task;
use iced_headless::Task as IcedTask;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    app::{
        CargoMakeMsg,
        cargo_make::{
            SettingsUpdate, Ui, UiMessage,
            command::{Command, Pinned},
        },
    },
    quick_pick::SelectInput,
    vs_code_api::showInformationMessage,
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

impl Ui {
    pub(crate) fn process_cmd(&self, cmd: Command) -> IcedTask<CargoMakeMsg> {
        match cmd {
            Command::RunTask(task) => IcedTask::done(CargoMakeMsg::Task(Task::from_string(task))),
            Command::SelectAndRunTask => {
                let input = SelectInput {
                    options: self.makefile_tasks.clone(),
                    current: Vec::new(),
                };
                run_task(
                    async move { input.select().await.map(|task| Command::RunTask(task.name)) },
                )
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
                    run_task(async move { input.select().await.map(Command::PinTask) })
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
