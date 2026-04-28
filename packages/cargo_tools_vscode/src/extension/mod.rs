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
    runtime::CHANNEL_CAPACITY,
    vs_code_api::{log_error, log_info, register_command},
};

pub type VsCodeTask = Closure<dyn FnMut(Array)>;
type TaskMap = HashMap<&'static str, Closure<dyn FnMut(Array)>>;

pub type OnFileChanged = Closure<dyn FnMut()>;

pub fn register_tasks(cmds: TaskMap) -> Vec<VsCodeTask> {
    cmds.into_iter()
        .map(|(command_id, cmd)| {
            log_info(&format!("Register task '{command_id}'"));
            if let Err(e) = register_command(command_id, &cmd) {
                log_error(&format!("Failed to register task '{command_id}': {e:?}"));
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
            log_error(&format!(
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
        log_info(&format!("Cargo tools extension received message:\n{msg:?}"));
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
            log_info("Signal to exit Cargo Tools extension");
            Subscription::run(|| {
                stream::channel(1, |mut tx: Sender<Exit>| async move {
                    if let Err(e) = tx.send(Exit).await {
                        log_error(&format!("Signal to exit Cargo Tools extension: {e}"));
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
            log_error(&format!("Error in Cargo Tools extension: {e}"));
        }
    });

    ExitToken(exit_tx)
}

fn init(root_dir: String, exit_rx: Receiver<()>) -> (Extension, Task<Message>) {
    log_info("Initializing Cargo tools");

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

    log_info("Finished initializing Cargo tools");
    (ext, task)
}
