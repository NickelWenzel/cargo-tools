pub mod cargo;
pub mod cargo_make;

use std::sync::Mutex;

use cargo_tools::app::{
    self, App, AppMessage,
    cargo::{Cargo, CargoMessage},
    cargo_make::{CargoMake, CargoMakeMessage},
};
use futures::{
    SinkExt,
    channel::mpsc::{Sender, channel},
};
use iced_headless::{Subscription, Task, async_application, event_loop::Exit};
use once_cell::sync::Lazy;

use crate::{command::register_commands, runtime::VsCodeRuntime, vs_code_api};

static EXIT_TX: Lazy<Mutex<Sender<Exit>>> = Lazy::new(|| {
    let (tx, _) = channel(10);
    Mutex::new(tx)
});

pub static MSG_RX: Lazy<Mutex<async_broadcast::Receiver<Message>>> = Lazy::new(|| {
    let (_, rx) = async_broadcast::broadcast(10);
    Mutex::new(rx)
});

#[derive(Debug, Clone)]
pub enum Message {
    Cargo(app::cargo::ui::Message<cargo::Ui>),
    CargoMake(app::cargo_make::ui::Message<cargo_make::Ui>),
}

#[derive(Debug)]
struct Ui;

impl cargo_tools::app::Ui for Ui {
    type Cargo = cargo::Ui;
    type CargoMake = cargo_make::Ui;
}

pub fn run(workspace_root: String) {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = async_application(App::update::<VsCodeRuntime>)
            .exit_on(exit_on)
            .run_with(|| init(workspace_root))
            .await
        {
            vs_code_api::log(&format!("Error in Cargo Tools extension: {e}"));
        }
    });
}

pub async fn exit() {
    let mut tx = EXIT_TX.lock().unwrap().clone();
    if let Err(e) = tx.send(Exit).await {
        vs_code_api::log(&format!(
            "Failed to send exit signal to Cargo Tools extension: {e}"
        ));
    }
}

fn init(root_dir: String) -> (App<Ui>, Task<AppMessage<Ui>>) {
    let cargo_ui = cargo::Ui::new();
    let cargo_make_ui = cargo_make::Ui::new();

    let app = App {
        cargo: Cargo::new(root_dir.clone(), cargo_ui.clone()),
        cargo_make: CargoMake::new(root_dir.clone(), cargo_make_ui.clone()),
    };

    let cargo = Task::done(AppMessage::Cargo(CargoMessage::RootDirUpdate(
        root_dir.clone(),
    )));
    let cargo_make = Task::done(AppMessage::CargoMake(CargoMakeMessage::RootDirUpdate(
        root_dir,
    )));

    register_commands(MSG_RX.lock().unwrap().new_sender(), cargo_ui, cargo_make_ui);

    (app, Task::batch([cargo, cargo_make]))
}

fn exit_on(_: &App<Ui>) -> Subscription<Exit> {
    Subscription::run(|| {
        let (tx, rx) = channel::<Exit>(10);
        *EXIT_TX.lock().unwrap() = tx;
        rx
    })
}
