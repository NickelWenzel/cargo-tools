use std::sync::{Arc, RwLock};

use cargo_metadata::Metadata;

use crate::{
    cargo_tools::{SettingsHandler, UserInterface},
    configuration_handler::spawn_configuration_handler,
    environment::MakefileTasks,
    runtime::Runtime,
    state::StateHandler,
    DEFAULT_BUFFER_SIZE,
};

/// Handles for the application layer threads.
///
/// This struct holds thread handles for the state handler, settings handler,
/// and user interface, allowing management of the application layer lifecycle.
pub struct ApplicationHandles<RuntimeT: Runtime> {
    pub state_handle: RuntimeT::ThreadHandle,
    pub settings_handle: RuntimeT::ThreadHandle,
    pub user_interface_handle: RuntimeT::ThreadHandle,
}

/// Spawns the application layer of cargo-tools.
///
/// This function creates and spawns:
/// - A state configuration handler that manages application state
/// - A settings configuration handler that manages user settings
/// - A user interface that reacts to metadata, makefile tasks, state, and settings changes
///
/// The function creates internal broadcast channels for state and settings updates,
/// which are used for communication between components but not exposed to callers.
///
/// # Arguments
///
/// * `user_interface` - The user interface implementation to spawn
/// * `metadata_rx` - Receiver for cargo metadata updates from the environment layer
/// * `makefile_tasks_rx` - Receiver for makefile task updates from the environment layer
///
/// # Returns
///
/// `ApplicationHandles` containing thread handles for all spawned components
pub async fn spawn_application<RuntimeT, UserInterfaceT>(
    user_interface: UserInterfaceT,
    metadata_rx: async_broadcast::Receiver<Arc<RwLock<Metadata>>>,
    makefile_tasks_rx: async_broadcast::Receiver<Arc<RwLock<MakefileTasks>>>,
) -> ApplicationHandles<RuntimeT>
where
    RuntimeT: Runtime,
    UserInterfaceT: UserInterface<Runtime = RuntimeT>,
{
    let (_state_update_tx, state_update_rx) = async_broadcast::broadcast(DEFAULT_BUFFER_SIZE);
    let (state_tx, state_rx) = async_broadcast::broadcast(DEFAULT_BUFFER_SIZE);

    let state_handle =
        spawn_configuration_handler::<StateHandler, RuntimeT>(state_tx, state_update_rx);

    let (_settings_update_tx, settings_update_rx) = async_broadcast::broadcast(DEFAULT_BUFFER_SIZE);
    let (settings_tx, settings_rx) = async_broadcast::broadcast(DEFAULT_BUFFER_SIZE);

    let settings_handle =
        spawn_configuration_handler::<SettingsHandler, RuntimeT>(settings_tx, settings_update_rx);

    // Spawn user interface with all necessary receivers
    let user_interface_handle =
        user_interface.spawn(metadata_rx, makefile_tasks_rx, state_rx, settings_rx);

    ApplicationHandles {
        state_handle,
        settings_handle,
        user_interface_handle,
    }
}
