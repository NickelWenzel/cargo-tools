pub mod cargo;
pub mod cargo_make;

use std::{
    hash::Hash,
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
};

use async_broadcast::SendError;
use cargo_tools::{
    app::{
        App, AppMessage,
        cargo::{Cargo, CargoMessage, selection},
        cargo_make::{CargoMake, CargoMakeMessage},
    },
    runtime::Runtime,
};
use futures::{
    SinkExt, Stream,
    channel::mpsc::{Sender, channel},
};
use iced_headless::{Subscription, Task, async_application, event_loop::Exit};
use once_cell::sync::Lazy;
use pin_project::pin_project;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{app::cargo::command_stream, runtime::VsCodeRuntime, vs_code_api::log};

pub type CargoMsg = ::cargo_tools::app::cargo::ui::Message<
    <cargo::Ui as ::cargo_tools::app::cargo::ui::Ui>::CustomUpdate,
>;

pub trait IntoCargoMessage {
    fn into_cargo_msg(self) -> CargoMsg;
}

impl IntoCargoMessage for selection::Update {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Selection(self)
    }
}

impl IntoCargoMessage for cargo_tools::app::cargo::ui::Task {
    fn into_cargo_msg(self) -> CargoMsg {
        CargoMsg::Task(self)
    }
}

pub type CargoMakeMsg = ::cargo_tools::app::cargo_make::ui::Message<
    <cargo_make::Ui as ::cargo_tools::app::cargo_make::ui::Ui>::CustomUpdate,
>;

pub type SendResult<T> = Result<Option<T>, SendError<T>>;

static EXIT_TX: Lazy<Mutex<Sender<Exit>>> = Lazy::new(|| {
    let (tx, _) = channel(10);
    Mutex::new(tx)
});

#[pin_project]
pub struct StaticHashStream<T, H> {
    #[pin]
    stream: T,
    hash: H,
}

impl<T, H> StaticHashStream<T, H> {
    pub fn new(stream: T, hash: H) -> Self {
        Self { stream, hash }
    }
}

impl<T, HashableT: Hash> Hash for StaticHashStream<T, HashableT> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T: Stream, H> Stream for StaticHashStream<T, H> {
    type Item = T::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl<T: Clone, H: Clone> Clone for StaticHashStream<T, H> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            hash: self.hash.clone(),
        }
    }
}

#[derive(Debug)]
struct Ui;

impl cargo_tools::app::Ui for Ui {
    type Cargo = cargo::Ui;
    type CargoMake = cargo_make::Ui;
}

#[wasm_bindgen]
pub fn run(workspace_root: String) {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = async_application(App::update::<VsCodeRuntime>)
            .subscription(App::subscription::<VsCodeRuntime>)
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
    let cargo_ui = cargo::Ui::new(
        VsCodeRuntime::get_state(format!("{root_dir}.cargo_tools.cargo.state")).unwrap_or_default(),
    );
    let cargo_make_ui = cargo_make::Ui::new(
        VsCodeRuntime::get_state(format!("{root_dir}.cargo_tools.cargo_make.state"))
            .unwrap_or_default(),
    );

    let app = App {
        cargo: Cargo::new(root_dir.clone(), cargo_ui),
        cargo_make: CargoMake::new(root_dir.clone(), cargo_make_ui),
    };

    let cargo = Task::done(AppMessage::Cargo(CargoMessage::RootDirUpdate(
        root_dir.clone(),
    )));
    // let cargo_make = Task::done(AppMessage::CargoMake(CargoMakeMessage::RootDirUpdate(
    //     root_dir,
    // )));
    log("Done initializing Cargo tools");

    let cargo_cmds = Task::stream(command_stream())
        .map(CargoMessage::Ui)
        .map(AppMessage::Cargo);

    // (app, Task::batch([cargo, cargo_make]))
    (app, Task::batch([cargo, cargo_cmds]))
}

fn exit_on(_: &App<Ui>) -> Subscription<Exit> {
    Subscription::run(|| {
        let (tx, rx) = channel::<Exit>(10);
        *EXIT_TX.lock().unwrap() = tx;
        rx
    })
}
