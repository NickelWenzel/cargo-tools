pub mod cargo;
pub mod cargo_make;

use std::{collections::HashMap, sync::Mutex};

use async_broadcast::SendError;
use cargo_tools::app::{App, AppMessage, cargo::CargoMessage, cargo_make::CargoMakeMessage};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_headless::{Subscription, Task, async_application, event_loop::Exit};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    runtime::VsCodeRuntime as Runtime,
    vs_code_api::{log, register_command},
};

pub type CargoMsg = ::cargo_tools::app::cargo::ui::Message<
    <cargo::Ui as ::cargo_tools::app::cargo::ui::Ui>::CustomUpdate,
>;

pub type VsCodeTask = Closure<dyn FnMut(Array)>;
type TaskMap = HashMap<&'static str, Closure<dyn FnMut(Array)>>;

pub fn register_tasks(cmds: TaskMap) -> Vec<VsCodeTask> {
    cmds.into_iter()
        .map(|(command_id, cmd)| {
            if let Err(e) = register_command(command_id, &cmd) {
                log(&format!(
                    "Failed to register task '{}': {:?}",
                    command_id, e
                ));
            };
            cmd
        })
        .collect()
}

pub type CargoMakeMsg = ::cargo_tools::app::cargo_make::ui::Message<
    <cargo_make::Ui as ::cargo_tools::app::cargo_make::ui::Ui>::CustomUpdate,
>;

pub type SendResult<T> = Result<Option<T>, SendError<T>>;

static EXIT_TX: Lazy<Mutex<Sender<Exit>>> = Lazy::new(|| {
    let (tx, _) = channel(10);
    Mutex::new(tx)
});

#[derive(Debug, Default)]
struct Ui;

impl cargo_tools::app::Ui for Ui {
    type Cargo = cargo::Ui;
    type CargoMake = cargo_make::Ui;
}

#[wasm_bindgen]
pub fn run(workspace_root: String) {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = async_application(App::update::<Runtime>)
            .subscription(App::subscription::<Runtime>)
            .exit_on(exit_on)
            .run_with(|| init(workspace_root))
            .await
        {
            log(&format!("Error in Cargo Tools extension: {e}"));
        }
    });
}

#[wasm_bindgen]
pub async fn exit() {
    let mut tx = EXIT_TX.lock().unwrap().clone();
    if let Err(e) = tx.send(Exit).await {
        log(&format!(
            "Failed to send exit signal to Cargo Tools extension: {e}"
        ));
    }
}

fn init(root_dir: String) -> (App<Ui>, Task<AppMessage<Ui>>) {
    log("Initializing Cargo tools");

    let cargo = Task::done(AppMessage::Cargo(CargoMessage::RootDirUpdate(
        root_dir.clone(),
    )));
    let cargo_make = Task::done(AppMessage::CargoMake(CargoMakeMessage::RootDirUpdate(
        root_dir,
    )));
    let ret = (App::default(), Task::batch([cargo, cargo_make]));
    log("Done initializing Cargo tools");
    ret
}

fn exit_on(_: &App<Ui>) -> Subscription<Exit> {
    Subscription::run(|| {
        let (tx, rx) = channel::<Exit>(10);
        *EXIT_TX.lock().unwrap() = tx;
        rx
    })
}

#[cfg(test)]
pub mod tests {
    use futures::channel::mpsc::channel;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        app::{
            cargo::command::task_map as cargo_task_map,
            cargo_make::command::task_map as cargo_make_task_map,
        },
        contributes::data::all_commands,
    };

    #[wasm_bindgen_test]
    fn all_commands_are_registered() {
        let (cargo_tx, _rx) = channel(10);
        let (cargo_make_tx, _rx) = channel(10);
        let closures = {
            let mut cmds = cargo_task_map(cargo_tx);
            cmds.extend(cargo_make_task_map(cargo_make_tx));
            cmds
        };
        let commands = all_commands();

        for cmd in commands {
            assert!(
                closures.contains_key(cmd.command.as_str()),
                "Command '{}' from all_commands() was not registered in COMMAND_CLOSURES",
                cmd.command
            );
        }
    }
}
