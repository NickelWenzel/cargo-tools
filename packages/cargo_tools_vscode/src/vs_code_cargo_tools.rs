//! VS Code-specific implementations of cargo-tools traits.
//!
//! This module contains concrete implementations of the traits defined in the
//! cargo_tools crate, adapted for the VS Code extension environment.

use cargo_tools::cargo_tools::CargoTools;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::vs_code_state_manager::VSCodeStateManager;

mod context_manager;
mod metadata_provider;
mod user_interface;
mod workspace;

// Re-export for convenience
pub use context_manager::VSCodeContextManager;
pub use metadata_provider::VSCodeMetaDataProvider;
pub use user_interface::VSCodeUserInterface;
pub use workspace::VSCodeWorkspace;

/// WASM wrapper around CargoTools for VS Code integration.
///
/// This struct provides the wasm_bindgen interface to the core CargoTools logic,
/// using VS Code-specific implementations of the required traits.
#[wasm_bindgen]
pub struct VSCodeCargoTools {
    inner: CargoTools<
        VSCodeMetaDataProvider,
        VSCodeWorkspace,
        VSCodeContextManager,
        VSCodeUserInterface,
    >,
}

#[wasm_bindgen]
impl VSCodeCargoTools {
    /// Create a new VSCodeCargoTools instance.
    ///
    /// This initializes cargo-tools with VS Code-specific implementations
    /// of all required traits.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (e.g., not a valid Cargo project).
    #[wasm_bindgen]
    pub async fn create() -> Result<VSCodeCargoTools, JsValue> {
        let metadata_provider = VSCodeMetaDataProvider;
        let workspace = VSCodeWorkspace::new();
        let state_manager = VSCodeStateManager::new();
        let context_manager = VSCodeContextManager::new(state_manager);
        let user_interface = VSCodeUserInterface::new();

        match CargoTools::create(
            metadata_provider,
            workspace,
            context_manager,
            user_interface,
        )
        .await
        {
            Ok(inner) => Ok(VSCodeCargoTools { inner }),
            Err(e) => {
                // Log error and return JsValue
                crate::vs_code_api::log(&format!("Failed to create CargoTools: {}", e));
                Err(JsValue::from_str(&e.to_string()))
            }
        }
    }
}
