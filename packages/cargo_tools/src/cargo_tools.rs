use std::sync::{Arc, RwLock};

use cargo_metadata::Metadata;
use wasm_async_trait::wasm_async_trait;

use crate::{
    application::{spawn_application, ApplicationHandles},
    configuration_handler::ConfigurationManager,
    environment::{spawn_environment, EnvironmentHandles, MakefileTasks},
    runtime::Runtime,
    state::State,
    DEFAULT_BUFFER_SIZE,
};

#[derive(Debug, Clone)]
pub struct Settings;

#[derive(Debug, Clone)]
pub struct SettingsUpdate;

pub struct SettingsHandler;

#[wasm_async_trait]
impl ConfigurationManager for SettingsHandler {
    type Configuration = Arc<RwLock<Settings>>;
    type ConfigurationUpdate = SettingsUpdate;

    async fn update_root_dir<RuntimeT: Runtime>(root_dir: String) -> Self::Configuration {
        Arc::new(RwLock::new(
            RuntimeT::update_settings_context(root_dir).await,
        ))
    }
    async fn apply_update<RuntimeT: Runtime>(update: SettingsUpdate) -> Self::Configuration {
        Arc::new(RwLock::new(RuntimeT::update_settings(update).await))
    }
}

#[wasm_async_trait]
pub trait UserInterface: Sized {
    type Runtime: Runtime;

    fn spawn(
        self,
        mut metadata_rx: async_broadcast::Receiver<Arc<RwLock<Metadata>>>,
        mut makefile_rx: async_broadcast::Receiver<Arc<RwLock<MakefileTasks>>>,
        mut state_rx: async_broadcast::Receiver<Arc<RwLock<State>>>,
        mut settings_rx: async_broadcast::Receiver<Arc<RwLock<Settings>>>,
    ) -> <Self::Runtime as Runtime>::ThreadHandle {
        Self::Runtime::spawn(async move {
            while let Ok(()) = match futures::future::select(
                futures::future::select(metadata_rx.recv(), makefile_rx.recv()),
                futures::future::select(state_rx.recv(), settings_rx.recv()),
            )
            .await
            {
                futures::future::Either::Left((select, _)) => match select {
                    futures::future::Either::Left((metadata, _)) => match metadata {
                        Ok(metadata) => {
                            let _: () = Self::update_metadata(metadata).await;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                    futures::future::Either::Right((makefile_tasks, _)) => match makefile_tasks {
                        Ok(makefile_tasks) => {
                            let _: () = Self::update_makefile_tasks(makefile_tasks).await;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                },
                futures::future::Either::Right((select, _)) => match select {
                    futures::future::Either::Left((state, _)) => match state {
                        Ok(state) => {
                            let _: () = Self::update_state(state).await;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                    futures::future::Either::Right((settings, _)) => match settings {
                        Ok(settings) => {
                            let _: () = Self::update_settings(settings).await;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                },
            } {
                // do nothing
            }
        })
    }

    async fn update_metadata(metadata: Arc<RwLock<Metadata>>);
    async fn update_makefile_tasks(makefile_tasks: Arc<RwLock<MakefileTasks>>);
    async fn update_state(state: Arc<RwLock<State>>);
    async fn update_settings(settings: Arc<RwLock<Settings>>);
}

pub struct CargoToolsHandles<RuntimeT: Runtime> {
    pub environment_handles: EnvironmentHandles<RuntimeT>,
    pub application_handles: ApplicationHandles<RuntimeT>,
}

pub async fn spawn_cargo_tools<RuntimeT, UserInterfaceT>(
    user_interface: UserInterfaceT,
) -> CargoToolsHandles<RuntimeT>
where
    RuntimeT: Runtime,
    UserInterfaceT: UserInterface<Runtime = RuntimeT>,
{
    let (metadata_tx, metadata_rx) = async_broadcast::broadcast(DEFAULT_BUFFER_SIZE);
    let (makefile_tasks_tx, makefile_tasks_rx) = async_broadcast::broadcast(DEFAULT_BUFFER_SIZE);
    let environment_handles = spawn_environment::<RuntimeT>(metadata_tx, makefile_tasks_tx).await;

    let application_handles = spawn_application::<RuntimeT, UserInterfaceT>(
        user_interface,
        metadata_rx,
        makefile_tasks_rx,
    )
    .await;

    CargoToolsHandles {
        environment_handles,
        application_handles,
    }
}
