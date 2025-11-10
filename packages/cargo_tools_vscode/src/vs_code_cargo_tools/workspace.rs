/// VS Code-specific implementation of Workspace.
///
/// This struct provides access to the VS Code workspace API.
pub struct VSCodeWorkspace;

impl VSCodeWorkspace {
    /// Create a new VSCodeWorkspace instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VSCodeWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

impl cargo_tools::cargo_tools::WorkspaceHandler for VSCodeWorkspace {
    fn get_root(&self) -> String {
        todo!("Get workspace root from VS Code API")
    }

    fn on_changed(&self, _on_changed: impl std::ops::AsyncFn()) {
        todo!("Register workspace change listener with VS Code API")
    }
}
