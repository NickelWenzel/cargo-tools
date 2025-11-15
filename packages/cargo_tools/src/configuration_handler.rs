use cargo_tools_macros::wasm_async_trait;

use crate::runtime::Runtime;

#[wasm_async_trait]
pub trait ConfigurationManager {
    type Configuration: Clone;
    type ConfigurationUpdate: Clone;

    async fn update_root_dir<RuntimeT: Runtime>(root_dir: String) -> Self::Configuration;
    async fn apply_update<RuntimeT: Runtime>(
        update: Self::ConfigurationUpdate,
    ) -> Self::Configuration;
}

pub fn spawn_configuration_handler<
    ConfigurationManagerT: ConfigurationManager,
    RuntimeT: Runtime,
>(
    config_tx: async_broadcast::Sender<ConfigurationManagerT::Configuration>,
    mut update_rx: async_broadcast::Receiver<ConfigurationManagerT::ConfigurationUpdate>,
) -> RuntimeT::ThreadHandle {
    RuntimeT::spawn(async move {
        let mut workspace_root_rx = RuntimeT::current_dir_notitifier().await;

        while let Ok(state) =
            match futures::future::select(workspace_root_rx.recv(), update_rx.recv()).await {
                futures::future::Either::Left((root_dir, _)) => match root_dir {
                    Ok(root_dir) => {
                        Ok(ConfigurationManagerT::update_root_dir::<RuntimeT>(root_dir).await)
                    }
                    Err(e) => Err(e),
                },
                futures::future::Either::Right((update, _)) => match update {
                    Ok(update) => Ok(ConfigurationManagerT::apply_update::<RuntimeT>(update).await),
                    Err(e) => Err(e),
                },
            }
        {
            let _ = config_tx.broadcast(state).await;
        }
    })
}
