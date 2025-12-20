use async_broadcast::Receiver;
use wasm_async_trait::wasm_async_trait;

#[wasm_async_trait]
pub trait Runtime: 'static {
    async fn exec(command: String) -> Result<String, String>;
    async fn log(msg: String);

    fn current_dir_notitifier() -> Receiver<String>;
    fn file_changed_notifier(file: String) -> Receiver<()>;
}
