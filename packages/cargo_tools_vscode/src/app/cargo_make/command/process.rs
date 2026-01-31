// mod project_outline;

use cargo_tools::{app::cargo_make::ui::Maketask, runtime::Runtime as _};
use iced_headless::Task;
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    app::cargo_make::{
        Message, SettingsUpdate, Ui,
        command::{Command, Pinned},
    },
    quick_pick::SelectInput,
    runtime::VsCodeRuntime as Runtime,
    vs_code_api::showInformationMessage,
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
            Command::RunTask(task) => self.make_task_exec(Maketask::from_string(task)),
            Command::SelectAndRunTask => {
                let input = SelectInput {
                    options: self.makefile_tasks.clone(),
                    current: Vec::new(),
                };
                done(async move { input.select().await.map(|task| Command::RunTask(task.name)) })
            }
            Command::SelectTaskFilter => todo!(),
            Command::EditTaskFilter(filter) => {
                Task::done(SettingsUpdate::TaskFilter(filter).into_cargo_make_msg())
            }
            Command::SelectCategoryFilter => todo!(),
            Command::EditCategoryFilter(filters) => {
                Task::done(SettingsUpdate::CategoryFilter(filters).into_cargo_make_msg())
            }
            Command::ClearAllFilters => {
                let task = Task::done(Command::EditTaskFilter(String::new()));
                let category = Task::done(Command::EditCategoryFilter(Vec::new()));
                Task::batch([task, category]).map(Command::into_cargo_make_msg)
            }
            Command::PinTask(task) => {
                Task::done(SettingsUpdate::AddPinned(task).into_cargo_make_msg())
            }
            Command::Pinned(pinned) => match pinned {
                Pinned::Add => {
                    let input = SelectInput {
                        options: self.makefile_tasks.clone(),
                        current: Vec::new(),
                    };
                    done(async move { input.select().await.map(Command::PinTask) })
                }
                Pinned::Remove(idx) => {
                    Task::done(SettingsUpdate::RemovePinned(idx).into_cargo_make_msg())
                }
                Pinned::Execute(task) => self.make_task_exec(Maketask::from_string(task)),
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
            Some(task) => self.make_task_exec(Maketask::from_string(task.name.clone())),
            None => Task::future(showInformationMessage(
                format!("There is no task no. {} pinned ", idx + 1),
                Array::new(),
            ))
            .discard(),
        }
    }

    fn make_task_exec(&self, make_task: Maketask) -> Task<Message> {
        let task = make_task.into_task(&Runtime::get_configuration());
        Task::future(Runtime::exec_task(task)).discard()
    }
}

fn done(
    fut: impl Future<Output = Option<impl IntoCargoMakeMessage + 'static>> + 'static,
) -> Task<Message> {
    Task::future(fut)
        .and_then(Task::done)
        .map(IntoCargoMakeMessage::into_cargo_make_msg)
}
