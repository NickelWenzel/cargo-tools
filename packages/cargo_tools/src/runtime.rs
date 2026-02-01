use std::{collections::HashMap, fmt::Debug};

use serde::{Serialize, de::DeserializeOwned};
use wasm_async_trait::wasm_async_trait;

use crate::configuration::Configuration;

pub enum CargoTask {
    Cargo(Task),
    CargoMake(Task),
    RustUp(Task),
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
    fn log(msg: String);

    async fn read_file(file_path: String) -> Result<String, String>;

    async fn persist_state(key: String, state: impl Serialize + Send);
    fn get_state<T: DeserializeOwned + Debug>(key: String) -> Option<T>;

    fn get_configuration() -> impl Configuration;
}
