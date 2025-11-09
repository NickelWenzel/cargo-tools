use cargo_metadata::Metadata;

use crate::vs_code_cargo_tools::context_manager::VSCodeContextManager;

/// VS Code-specific implementation of UserInterface.
///
/// This struct manages the VS Code extension UI elements (tree views, status bar, etc.).
pub struct VSCodeUserInterface;

impl VSCodeUserInterface {
    /// Create a new VSCodeUserInterface instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VSCodeUserInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl cargo_tools::cargo_tools::UserInterface for VSCodeUserInterface {
    type ContextManagerT = VSCodeContextManager;

    fn init(&self, _metadata: &Metadata, _context_manager: &Self::ContextManagerT) {
        todo!("Initialize VS Code UI elements with cargo metadata")
    }

    fn reset(&self) {
        todo!("Reset VS Code UI elements")
    }
}
