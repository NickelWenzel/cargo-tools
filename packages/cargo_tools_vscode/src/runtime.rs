use cargo_tools::runtime::Runtime;
use futures::channel::mpsc::Receiver;
use wasm_async_trait::wasm_async_trait;

pub struct VsCodeRuntime;

#[wasm_async_trait]
impl Runtime for VsCodeRuntime {
    async fn exec(_command: String) -> Result<String, String> {
        todo!()
    }

    async fn log(_msg: String) {
        todo!()
    }

    fn current_dir_notitifier() -> Receiver<String> {
        todo!()
    }

    fn file_changed_notifier(_file: String) -> Receiver<()> {
        todo!()
    }
}
