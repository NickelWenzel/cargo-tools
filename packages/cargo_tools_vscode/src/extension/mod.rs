pub mod base;
pub mod cargo;
pub mod cargo_make;

use std::collections::HashMap;

use futures::{
    SinkExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use iced_headless::{Subscription, Task, async_application, event_loop::Exit, stream};
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use wasm_bindgen_futures::js_sys::Array;

use crate::{
    runtime::{CHANNEL_CAPACITY, log_vs_code},
    vs_code_api::{log, register_command},
};

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

#[wasm_bindgen]
pub struct ExitToken(Sender<()>);

#[wasm_bindgen]
impl ExitToken {
    #[wasm_bindgen]
    pub async fn exit(&mut self) {
        if let Err(e) = self.0.send(()).await {
            log(&format!(
                "Failed to signal exit to Cargo Tools extension: {e:?}"
            ));
        }
    }
}

#[derive(Debug)]
enum Message {
    Cargo(cargo::Message),
    CargoMake(cargo_make::Message),
    Exit,
}

struct Extension {
    cargo: cargo::Ui,
    cargo_make: cargo_make::Ui,
    exit: bool,
}

impl Extension {
    fn update(&mut self, msg: Message) -> Task<Message> {
        log_vs_code(format!("Cargo tools extension received message:\n{msg:?}"));
        match msg {
            Message::Cargo(msg) => self.cargo.update(msg).map(Message::Cargo),
            Message::CargoMake(msg) => self.cargo_make.update(msg).map(Message::CargoMake),
            Message::Exit => {
                self.exit = true;
                Task::none()
            }
        }
    }

    fn exit(&self) -> Subscription<Exit> {
        if self.exit {
            log("Signal to exit Cargo Tools extension");
            Subscription::run(|| {
                stream::channel(1, |mut tx: Sender<Exit>| async move {
                    if let Err(e) = tx.send(Exit).await {
                        log(&format!("Signal to exit Cargo Tools extension: {e}"));
                    };
                })
            })
        } else {
            Subscription::none()
        }
    }
}

#[wasm_bindgen]
pub fn run(workspace_root: String) -> ExitToken {
    let (exit_tx, exit_rx) = channel(CHANNEL_CAPACITY);
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = async_application(Extension::update)
            .exit_on(Extension::exit)
            .run_with(|| init(workspace_root, exit_rx))
            .await
        {
            log(&format!("Error in Cargo Tools extension: {e}"));
        }
    });

    ExitToken(exit_tx)
}

fn init(root_dir: String, exit_rx: Receiver<()>) -> (Extension, Task<Message>) {
    log("Initializing Cargo tools");

    let (cargo, cargo_task) = cargo::Ui::new(root_dir.clone());
    let (cargo_make, cargo_make_task) = cargo_make::Ui::new(root_dir.clone());

    let ext = Extension {
        cargo,
        cargo_make,
        exit: false,
    };

    let task = Task::batch([
        cargo_task.map(Message::Cargo),
        cargo_make_task.map(Message::CargoMake),
        Task::stream(exit_rx).map(|()| Message::Exit),
    ]);

    log("Finished initializing Cargo tools");
    (ext, task)
}

// #[cfg(test)]
// pub mod tests {
//     use wasm_bindgen_test::wasm_bindgen_test;

//     use crate::extension::{
//         cargo::{self},
//         cargo_make::{self},
//     };

//     fn all_commands() -> Vec<String> {
//         let package_json = include_str!("../../../../package.json");
//         let json: serde_json::Value =
//             serde_json::from_str(package_json).expect("Failed to parse package.json");

//         json["contributes"]["commands"]
//             .as_array()
//             .expect("commands should be an array")
//             .iter()
//             .filter_map(|cmd| cmd["command"].as_str().map(|s| s.to_string()))
//             .collect()
//     }

//     #[wasm_bindgen_test]
//     fn all_commands_are_registered() {
//         let all_keys = {
//             let mut cmds = cargo::command::all_keys().into_iter().collect::<Vec<_>>();
//             cmds.extend(cargo_make::command::all_keys());
//             cmds
//         };
//         let commands = all_commands();

//         for cmd in commands {
//             assert!(
//                 all_keys.contains(&cmd.as_str()),
//                 "Command '{cmd}' from all_commands() was not registered in COMMAND_CLOSURES."
//             );
//         }
//     }
// }
