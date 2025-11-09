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

    fn set_is_cargo_project(&self, _is_cargo_project: bool) {
        todo!("Set cargo project context in VS Code")
    }

    fn set_has_makefile(&self, _has_makefile: bool) {
        todo!("Set makefile context in VS Code")
    }

    fn state_manager(&self) -> &Self::StateManagerT {
        &self.state_manager
    }
}
