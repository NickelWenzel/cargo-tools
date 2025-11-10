use cargo_tools_macros::wasm_async_trait;

use crate::cargo_tools::Runtime;

#[wasm_async_trait]
pub trait ConfigurationManager: Sized {
    type Runtime: Runtime;

    type Configuration: Clone;
    type ConfigurationUpdate: Clone;

    fn spawn(
        self,
        config_tx: async_broadcast::Sender<Self::Configuration>,
        mut root_dir_rx: async_broadcast::Receiver<String>,
        mut update_rx: async_broadcast::Receiver<Self::ConfigurationUpdate>,
    ) -> <Self::Runtime as Runtime>::ThreadHandle {
        Self::Runtime::spawn(async move {
            while let Ok(state) =
                match futures::future::select(root_dir_rx.recv(), update_rx.recv()).await {
                    futures::future::Either::Left((root_dir, _)) => match root_dir {
                        Ok(root_dir) => Ok(Self::update_root_dir(root_dir).await),
                        Err(e) => Err(e),
                    },
                    futures::future::Either::Right((update, _)) => match update {
                        Ok(update) => Ok(Self::apply_update(update).await),
                        Err(e) => Err(e),
                    },
                }
            {
                let _ = config_tx.broadcast(state).await;
            }
        })
    }

    async fn update_root_dir(root_dir: String) -> Self::Configuration;
    async fn apply_update(event: Self::ConfigurationUpdate) -> Self::Configuration;
}
