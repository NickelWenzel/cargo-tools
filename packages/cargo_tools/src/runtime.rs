use std::future::Future;

use async_broadcast::Receiver;
use cargo_tools_macros::wasm_async_trait;

#[wasm_async_trait]
pub trait Runtime {
    type ThreadHandle: Future;

    fn spawn<Result, F>(f: F) -> Self::ThreadHandle
    where
        Self::ThreadHandle: Future<Output = Result>;

    async fn exec(command: String) -> Result<String, String>;
    async fn log(msg: String);

    async fn current_dir_notitifier() -> Receiver<String>;
}
