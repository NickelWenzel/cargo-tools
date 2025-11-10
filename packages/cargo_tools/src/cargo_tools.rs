use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, RwLock},
};

use cargo_metadata::Metadata;
use cargo_tools_macros::wasm_async_trait;
use futures::{Stream, StreamExt};

use crate::{
    configuration_handler::ConfigurationManager,
    state::{State, StateUpdate},
};

pub trait Runtime {
    type ThreadHandle: Future;

    fn spawn<Result, F>(f: F) -> Self::ThreadHandle
    where
        Self::ThreadHandle: Future<Output = Result>;
}

#[derive(Debug, Clone)]
pub enum MetadataUpdate {
    New(Arc<RwLock<Metadata>>),
    NoCargoToml,
    FailedToRetrieve,
}

#[wasm_async_trait]
pub trait CargoTomlHandler: Sized {
    type Runtime: Runtime;

    fn spawn(
        self,
        metadata_tx: async_broadcast::Sender<Arc<RwLock<Metadata>>>,
        mut manifest_dir_rx: async_broadcast::Receiver<String>,
    ) -> <Self::Runtime as Runtime>::ThreadHandle {
        Self::Runtime::spawn(async move {
            while let Some(manifest_dir) = manifest_dir_rx.next().await {
                match self.update_metadata(&manifest_dir).await {
                    MetadataUpdate::New(metadata) => {
                        let _ = metadata_tx.broadcast(metadata).await;
                    }
                    MetadataUpdate::NoCargoToml | MetadataUpdate::FailedToRetrieve => {}
                }
            }
        })
    }

    async fn update_metadata(&self, path: &str) -> MetadataUpdate;
}

#[derive(Debug, Clone)]
pub struct MakefileTask {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct MakefileTaskCategory;

pub type MakefileTasks = HashMap<MakefileTaskCategory, MakefileTask>;

#[derive(Debug, Clone)]
pub enum MakefileTasksUpdate {
    New(Arc<RwLock<MakefileTasks>>),
    NoMakefile,
    FailedToRetrieve,
}

#[wasm_async_trait]
pub trait MakefileHandler: Sized {
    type Runtime: Runtime;

    fn spawn(
        self,
        makefile_tasks_tx: async_broadcast::Sender<Arc<RwLock<MakefileTasks>>>,
        mut manifest_dir_rx: async_broadcast::Receiver<String>,
    ) -> <Self::Runtime as Runtime>::ThreadHandle {
        Self::Runtime::spawn(async move {
            while let Some(manifest_dir) = manifest_dir_rx.next().await {
                match self.update_makefile_tasks(&manifest_dir).await {
                    MakefileTasksUpdate::New(makefile_tasks) => {
                        let _ = makefile_tasks_tx.broadcast(makefile_tasks).await;
                    }
                    MakefileTasksUpdate::NoMakefile | MakefileTasksUpdate::FailedToRetrieve => {}
                }
            }
        })
    }

    async fn update_makefile_tasks(&self, path: &str) -> MakefileTasksUpdate;
}

#[wasm_async_trait]
pub trait WorkspaceHandler: Sized {
    type Runtime: Runtime;

    fn spawn(
        self,
        workspace_root_tx: async_broadcast::Sender<String>,
    ) -> <Self::Runtime as Runtime>::ThreadHandle {
        Self::Runtime::spawn(async move {
            let mut workspace_root_stream = Box::pin(self.into_workspace_root_stream().await);
            while let Some(workspace_root) = workspace_root_stream.next().await {
                let _ = workspace_root_tx.broadcast(workspace_root).await;
            }
        })
    }

    async fn into_workspace_root_stream(self) -> impl Stream<Item = String>;
}

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
    pub workspace_handle: RuntimeT::ThreadHandle,
    pub cargo_toml_handle: RuntimeT::ThreadHandle,
    pub makefile_handle: RuntimeT::ThreadHandle,
    pub state_handle: RuntimeT::ThreadHandle,
    pub settings_handle: RuntimeT::ThreadHandle,
    pub user_interface_handle: RuntimeT::ThreadHandle,
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_cargo_tools<
    RuntimeT,
    WorkspaceHandlerT,
    CargoTomlHandlerT,
    MakefileHandlerT,
    StateHandlerT,
    SettingsHandlerT,
    UserInterfaceT,
>(
    workspace_handler: WorkspaceHandlerT,
    cargo_toml_handler: CargoTomlHandlerT,
    makefile_handler: MakefileHandlerT,
    state_handler: StateHandlerT,
    settings_handler: SettingsHandlerT,
    user_interface: UserInterfaceT,
    state_update_rx: async_broadcast::Receiver<StateUpdate>,
    settings_update_rx: async_broadcast::Receiver<SettingsUpdate>,
) -> CargoToolsHandles<RuntimeT>
where
    RuntimeT: Runtime,
    WorkspaceHandlerT: WorkspaceHandler<Runtime = RuntimeT>,
    CargoTomlHandlerT: CargoTomlHandler<Runtime = RuntimeT>,
    MakefileHandlerT: MakefileHandler<Runtime = RuntimeT>,
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
    let (workspace_root_tx, workspace_root_rx) = async_broadcast::broadcast(100);
    let workspace_handle = workspace_handler.spawn(workspace_root_tx);

    let (metadata_tx, metadata_rx) = async_broadcast::broadcast(100);
    let cargo_toml_handle = cargo_toml_handler.spawn(metadata_tx, workspace_root_rx.clone());

    let (makefile_tasks_tx, makefile_tasks_rx) = async_broadcast::broadcast(100);
    let makefile_handle = makefile_handler.spawn(makefile_tasks_tx, workspace_root_rx.clone());

    let (state_tx, state_rx) = async_broadcast::broadcast(100);
    let state_handle = state_handler.spawn(state_tx, workspace_root_rx.clone(), state_update_rx);

    let (settings_tx, settings_rx) = async_broadcast::broadcast(100);
    let settings_handle =
        settings_handler.spawn(settings_tx, workspace_root_rx, settings_update_rx);

    let user_interface_handle =
        user_interface.spawn(metadata_rx, makefile_tasks_rx, state_rx, settings_rx);

    CargoToolsHandles {
        workspace_handle,
        cargo_toml_handle,
        makefile_handle,
        state_handle,
        settings_handle,
        user_interface_handle,
    }
}
