use std::sync::{Arc, RwLock};

use cargo_metadata::Metadata;
use cargo_tools_macros::wasm_async_trait;

use crate::{
    configuration_handler::ConfigurationManager,
    environment::{spawn_environment, EnvironmentHandles, MakefileTasks},
    runtime::Runtime,
    state::{State, StateUpdate},
};

#[derive(Debug, Clone)]
pub struct Settings;

#[derive(Debug, Clone)]
pub struct SettingsUpdate;

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
    pub state_handle: RuntimeT::ThreadHandle,
    pub settings_handle: RuntimeT::ThreadHandle,
    pub user_interface_handle: RuntimeT::ThreadHandle,
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_cargo_tools<RuntimeT, StateHandlerT, SettingsHandlerT, UserInterfaceT>(
    state_handler: StateHandlerT,
    settings_handler: SettingsHandlerT,
    user_interface: UserInterfaceT,
    state_update_rx: async_broadcast::Receiver<StateUpdate>,
    settings_update_rx: async_broadcast::Receiver<SettingsUpdate>,
) -> CargoToolsHandles<RuntimeT>
where
    RuntimeT: Runtime,
    StateHandlerT: ConfigurationManager<
        Runtime = RuntimeT,
        Configuration = Arc<RwLock<State>>,
        ConfigurationUpdate = StateUpdate,
    >,
    SettingsHandlerT: ConfigurationManager<
        Runtime = RuntimeT,
        Configuration = Arc<RwLock<Settings>>,
        ConfigurationUpdate = SettingsUpdate,
    >,
    UserInterfaceT: UserInterface<Runtime = RuntimeT>,
{
    let (_workspace_root_tx, workspace_root_rx) = async_broadcast::broadcast(100);
    let (metadata_tx, metadata_rx) = async_broadcast::broadcast(100);
    let (makefile_tasks_tx, makefile_tasks_rx) = async_broadcast::broadcast(100);
    let environment_handles = spawn_environment::<RuntimeT>(metadata_tx, makefile_tasks_tx).await;

    let (state_tx, state_rx) = async_broadcast::broadcast(100);
    let state_handle = state_handler.spawn(state_tx, workspace_root_rx.clone(), state_update_rx);

    let (settings_tx, settings_rx) = async_broadcast::broadcast(100);
    let settings_handle =
        settings_handler.spawn(settings_tx, workspace_root_rx, settings_update_rx);

    let user_interface_handle =
        user_interface.spawn(metadata_rx, makefile_tasks_rx, state_rx, settings_rx);

    CargoToolsHandles {
        environment_handles,
        state_handle,
        settings_handle,
        user_interface_handle,
    }
}
