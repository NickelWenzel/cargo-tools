use std::{collections::HashMap, fmt::Debug};

use async_broadcast::Receiver;
use serde::{de::DeserializeOwned, Serialize};
use wasm_async_trait::wasm_async_trait;

use crate::configuration::Configuration;

pub enum CargoTask {
    Cargo(Task),
    CargoMake(Task),
}

pub struct Task {
    pub cmd: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[wasm_async_trait]
pub trait Runtime: 'static {
    async fn exec(command: String) -> Result<String, String>;
    async fn exec_task(task: CargoTask);
    async fn log(msg: String);

    fn current_dir_notitifier() -> Receiver<String>;
    fn file_changed_notifier(file: String) -> Receiver<()>;

    async fn persist_state(key: String, state: impl Serialize + Send);
    fn get_state<T: DeserializeOwned + Debug>(key: String) -> Option<T>;

    fn get_configuration() -> Option<impl Configuration>;
}
