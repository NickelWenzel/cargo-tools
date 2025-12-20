use futures::channel::mpsc::Receiver;
use wasm_async_trait::wasm_async_trait;

#[derive(Debug, Clone)]
pub struct Settings;

#[derive(Debug, Clone)]
pub struct SettingsUpdate;

#[wasm_async_trait]
pub trait Runtime: 'static {
    async fn exec(command: String) -> Result<String, String>;
    async fn log(msg: String);

    fn current_dir_notitifier() -> Receiver<String>;
    fn file_changed_notifier(file: String) -> Receiver<()>;
}
