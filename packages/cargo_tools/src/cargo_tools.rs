use std::error::Error;

use cargo_metadata::Metadata;

use crate::state_manager::StateManager;

pub trait MetaDataProvider {
    type Error: Error + 'static;

    fn request(&self, path: &str) -> Result<Metadata, Self::Error>;
}

pub trait Workspace {
    fn get_root(&self) -> String;
    fn on_changed(&self, on_changed: impl AsyncFn());
}

pub trait ContextManager {
    type StateManagerT;

    fn set_is_cargo_project(&self, is_cargo_project: bool);
    fn set_has_makefile(&self, is_makefile: bool);

    fn state_manager(&self) -> &Self::StateManagerT;
}

pub trait UserInterface {
    type ContextManagerT;

    fn init(&self, metadata: &Metadata, context_manager: &Self::ContextManagerT);
    fn reset(&self);
}

#[derive(Debug, thiserror::Error)]
pub enum InitError<MetaDataProviderT: MetaDataProvider> {
    #[error("Failed to initialize Cargo Tools")]
    MetaDataError(#[source] MetaDataProviderT::Error),
}

pub struct CargoTools<MetaDataProviderT, WorkspaceT, ContextManagerT, UserInterfaceT> {
    metadata_provider: MetaDataProviderT,
    workspace: WorkspaceT,
    context_manager: ContextManagerT,
    user_interface: UserInterfaceT,
}

impl<
        MetaDataProviderT: MetaDataProvider,
        WorkspaceT: Workspace,
        ContextManagerT: ContextManager<StateManagerT: StateManager>,
        UserInterfaceT: UserInterface<ContextManagerT = ContextManagerT>,
    > CargoTools<MetaDataProviderT, WorkspaceT, ContextManagerT, UserInterfaceT>
{
    pub async fn create(
        metadata_provider: MetaDataProviderT,
        workspace: WorkspaceT,
        context_manager: ContextManagerT,
        user_interface: UserInterfaceT,
    ) -> Result<Self, InitError<MetaDataProviderT>> {
        let cargo_tools = Self {
            metadata_provider,
            workspace,
            context_manager,
            user_interface,
        };

        cargo_tools.workspace.on_changed(|| async {
            cargo_tools.reset().await;
            let _ = cargo_tools.init().await;
        });

        cargo_tools.init().await.map(|()| cargo_tools)
    }

    pub async fn init(&self) -> Result<(), InitError<MetaDataProviderT>> {
        let res = {
            let workspace_root = self.workspace.get_root();
            let cargo_metadata = self
                .metadata_provider
                .request(&workspace_root)
                .map_err(InitError::MetaDataError)?;

            self.user_interface
                .init(&cargo_metadata, &self.context_manager);

            Ok(())
        };

        match res {
            Ok(()) => {
                self.context_manager.set_is_cargo_project(true);
                Ok(())
            }
            Err(e) => {
                self.reset().await;
                Err(e)
            }
        }
    }

    pub async fn reset(&self) {
        self.user_interface.reset();

        self.context_manager.set_is_cargo_project(false);
        self.context_manager.set_has_makefile(false);
    }
}
