pub mod base;
pub mod cargo;
pub mod cargo_make;

use std::{collections::HashMap, sync::Mutex};

use async_broadcast::SendError;
use cargo_tools::runtime::Runtime as _;
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

pub type OnFileChanged = Closure<dyn FnMut()>;

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

#[derive(Debug)]
enum Message {
    Cargo(cargo::Message),
    CargoMake(cargo_make::Message),
}

use Message::Cargo;
use Message::CargoMake;

struct Extension {
    pub cargo: cargo::Ui,
    pub cargo_make: cargo_make::Ui,
}

impl Extension {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        Runtime::log(format!("Cargo tools extension received message:\n{msg:?}"));
        match msg {
            Cargo(msg) => self.cargo.update(msg).map(Cargo),
            CargoMake(msg) => self.cargo_make.update(msg).map(CargoMake),
        }
    }
}

#[wasm_bindgen]
pub fn run(workspace_root: String) {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = async_application(Extension::update)
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

fn init(root_dir: String) -> (Extension, Task<Message>) {
    log("Initializing Cargo tools");

    let (cargo, cargo_task) = cargo::Ui::new(root_dir.clone());
    let (cargo_make, cargo_make_task) = cargo_make::Ui::new(root_dir.clone());

    let ext = Extension { cargo, cargo_make };
    let task = Task::batch([cargo_task.map(Cargo), cargo_make_task.map(CargoMake)]);

    log("Done initializing Cargo tools");
    (ext, task)
}

fn exit_on(_: &Extension) -> Subscription<Exit> {
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
