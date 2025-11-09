use crate::vs_code_api;
use crate::vs_code_state_manager::VSCodeStateManager;

/// VS Code-specific implementation of ContextManager.
///
/// This struct manages the VS Code extension context and state.
pub struct VSCodeContextManager {
    state_manager: VSCodeStateManager,
}

impl VSCodeContextManager {
    /// Create a new VSCodeContextManager instance.
    pub fn new(state_manager: VSCodeStateManager) -> Self {
        Self { state_manager }
    }
}

impl cargo_tools::cargo_tools::ContextManager for VSCodeContextManager {
    type StateManagerT = VSCodeStateManager;

    fn set_is_cargo_project(&self, is_cargo_project: bool) {
        vs_code_api::set_cargo_context(is_cargo_project);
    }

    fn set_has_makefile(&self, has_makefile: bool) {
        vs_code_api::set_makefile_context(has_makefile);
    }

    fn state_manager(&self) -> &Self::StateManagerT {
        &self.state_manager
    }
}
