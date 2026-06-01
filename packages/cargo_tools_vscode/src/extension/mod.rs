pub mod vscode_task_utils;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub use vscode_task_utils::{CommandBinding, OnFileChanged, select_name_filter, send_file_changed};

pub mod workspace;
use workspace::Workspace;

pub mod tasks;
use tasks::Tasks;

use futures::{
    SinkExt,
    channel::mpsc::{Receiver, Sender, channel},
};
use iced_viewless::{Subscription, Task, async_application, event_loop::Exit, stream};
use wasm_bindgen::prelude::*;

use crate::{logger::VSCodeLogger, runtime::CHANNEL_CAPACITY};
use tracing::{error, info};

#[wasm_bindgen]
pub struct ExitToken(Sender<()>);

#[wasm_bindgen]
impl ExitToken {
    #[wasm_bindgen]
    pub async fn exit(&mut self) {
        if let Err(e) = self.0.send(()).await {
            error!("Failed to signal exit to Cargo Tools extension: {e}");
        }
    }
}

#[derive(Debug)]
enum Message {
    Workspace(workspace::Message),
    Tasks(tasks::Message),
    Exit,
}

struct Extension {
    workspace: Workspace,
    tasks: Tasks,
    exit: bool,
}

impl Extension {
    fn update(&mut self, msg: Message) -> Task<Message> {
        info!("Cargo tools extension received message:\n{msg:?}");
        match msg {
            Message::Workspace(msg) => self.workspace.update(msg).map(Message::Workspace),
            Message::Tasks(msg) => self.tasks.update(msg).map(Message::Tasks),
            Message::Exit => {
                self.exit = true;
                Task::none()
            }
        }
    }

    fn exit(&self) -> Subscription<Exit> {
        if self.exit {
            info!("Signal to exit Cargo Tools extension");
            Subscription::run(|| {
                stream::channel(1, |mut tx: Sender<Exit>| async move {
                    if let Err(e) = tx.send(Exit).await {
                        error!("Signal to exit Cargo Tools extension: {e}");
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
    tracing_subscriber::registry().with(VSCodeLogger).init();

    let (exit_tx, exit_rx) = channel(CHANNEL_CAPACITY);
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = async_application(Extension::update)
            .exit_on(Extension::exit)
            .run_with(|| init(workspace_root, exit_rx))
            .await
        {
            error!("Error in Cargo Tools extension: {e}");
        }
    });

    ExitToken(exit_tx)
}

fn init(root_dir: String, exit_rx: Receiver<()>) -> (Extension, Task<Message>) {
    info!("Initializing Cargo tools");

    let (workspace, workspace_task) = Workspace::init(root_dir.clone());
    let (tasks, tasks_task) = Tasks::init(root_dir.clone());

    let ext = Extension {
        workspace,
        tasks,
        exit: false,
    };

    let task = Task::batch([
        workspace_task.map(Message::Workspace),
        tasks_task.map(Message::Tasks),
        Task::stream(exit_rx).map(|()| Message::Exit),
    ]);

    info!("Finished initializing Cargo tools");
    (ext, task)
}
