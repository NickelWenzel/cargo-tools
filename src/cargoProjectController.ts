import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoConfigurationReader } from './cargoConfigurationReader';

/**
 * Project controller for managing multiple workspace folders with cargo projects.
 * This follows the pattern from microsoft/vscode-cmake-tools for multi-project support.
 */
export class CargoProjectController implements vscode.Disposable {
    private readonly folderToWorkspaceMap = new Map<vscode.WorkspaceFolder, CargoWorkspace>();
    private subscriptions: vscode.Disposable[] = [];
    private activeFolder?: vscode.WorkspaceFolder;

    constructor(
        private readonly extensionContext: vscode.ExtensionContext,
        private readonly workspaceConfig: CargoConfigurationReader
    ) {
        this.setupSubscriptions();
    }

    /**
     * Set up subscriptions for workspace events
     */
    private setupSubscriptions(): void {
        this.subscriptions = [
            vscode.workspace.onDidChangeWorkspaceFolders(
                e => this.handleWorkspaceFoldersChanged(e)
            ),
            
            vscode.workspace.onDidOpenTextDocument(
                document => this.handleDocumentOpened(document)
            ),
            
            vscode.workspace.onDidSaveTextDocument(
                document => this.handleDocumentSaved(document)
            ),

            vscode.workspace.onDidRenameFiles(
                e => this.handleFilesRenamed(e)
            )
        ];
    }

    /**
     * Load all workspace folders and discover cargo projects
     */
    async loadAllFolders(): Promise<void> {
        if (!vscode.workspace.workspaceFolders) {
            return;
        }

        for (const folder of vscode.workspace.workspaceFolders) {
            await this.addFolder(folder);
        }

        // Auto-select active folder if configured
        if (this.workspaceConfig.autoSelectActiveProject) {
            await this.autoSelectActiveFolder();
        }
    }

    /**
     * Add a workspace folder and create cargo workspace if it contains a Cargo.toml
     */
    async addFolder(folder: vscode.WorkspaceFolder): Promise<CargoWorkspace[]> {
        if (this.folderToWorkspaceMap.has(folder)) {
            return [this.folderToWorkspaceMap.get(folder)!];
        }

        // Check if folder should be excluded
        if (this.isFolderExcluded(folder)) {
            return [];
        }

        try {
            // Check if folder contains Cargo.toml
            const cargoTomlPath = vscode.Uri.joinPath(folder.uri, 'Cargo.toml');
            const hasCargoToml = await vscode.workspace.fs.stat(cargoTomlPath).then(() => true, () => false);
            
            if (hasCargoToml) {
                const cargoWorkspace = new CargoWorkspace(folder.uri.fsPath);
                await cargoWorkspace.initialize();
                this.folderToWorkspaceMap.set(folder, cargoWorkspace);
                
                console.log(`Added cargo workspace: ${folder.name}`);
                return [cargoWorkspace];
            }
        } catch (error) {
            console.error(`Failed to add cargo workspace for ${folder.name}:`, error);
        }

        return [];
    }

    /**
     * Remove a workspace folder
     */
    async removeFolder(folder: vscode.WorkspaceFolder): Promise<void> {
        const cargoWorkspace = this.folderToWorkspaceMap.get(folder);
        if (cargoWorkspace) {
            this.folderToWorkspaceMap.delete(folder);
            
            // If this was the active folder, select a new one
            if (this.activeFolder === folder) {
                await this.autoSelectActiveFolder();
            }
            
            console.log(`Removed cargo workspace: ${folder.name}`);
        }
    }

    /**
     * Get the cargo workspace for a specific folder
     */
    getWorkspaceForFolder(folder?: vscode.WorkspaceFolder): CargoWorkspace | undefined {
        if (!folder) {
            return this.getActiveWorkspace();
        }
        return this.folderToWorkspaceMap.get(folder);
    }

    /**
     * Get the currently active cargo workspace
     */
    getActiveWorkspace(): CargoWorkspace | undefined {
        if (this.activeFolder) {
            return this.folderToWorkspaceMap.get(this.activeFolder);
        }
        
        // Return first available workspace if no active folder
        const workspaces = Array.from(this.folderToWorkspaceMap.values());
        return workspaces.length > 0 ? workspaces[0] : undefined;
    }

    /**
     * Get all cargo workspaces
     */
    getAllWorkspaces(): CargoWorkspace[] {
        return Array.from(this.folderToWorkspaceMap.values());
    }

    /**
     * Set the active workspace folder
     */
    async setActiveFolder(folder: vscode.WorkspaceFolder): Promise<void> {
        if (this.folderToWorkspaceMap.has(folder)) {
            this.activeFolder = folder;
            console.log(`Active cargo folder set to: ${folder.name}`);
        }
    }

    /**
     * Check if there are multiple cargo projects
     */
    get hasMultipleProjects(): boolean {
        return this.folderToWorkspaceMap.size > 1;
    }

    /**
     * Auto-select the active folder based on configuration and context
     */
    private async autoSelectActiveFolder(): Promise<void> {
        if (this.folderToWorkspaceMap.size === 0) {
            this.activeFolder = undefined;
            return;
        }

        // If there's a default active project configured, try to use it
        const defaultProject = this.workspaceConfig.defaultActiveProject;
        if (defaultProject) {
            const folder = vscode.workspace.workspaceFolders?.find(f => f.name === defaultProject);
            if (folder && this.folderToWorkspaceMap.has(folder)) {
                this.activeFolder = folder;
                return;
            }
        }

        // Otherwise, use the first available workspace
        const folders = Array.from(this.folderToWorkspaceMap.keys());
        this.activeFolder = folders[0];
    }

    /**
     * Check if a folder should be excluded based on configuration
     */
    private isFolderExcluded(folder: vscode.WorkspaceFolder): boolean {
        const excludePatterns = this.workspaceConfig.excludeFolders;
        const folderPath = folder.uri.fsPath;
        
        return excludePatterns.some(pattern => {
            // Simple pattern matching - could be enhanced with glob patterns
            return folderPath.includes(pattern);
        });
    }

    /**
     * Handle workspace folders changed event
     */
    private async handleWorkspaceFoldersChanged(event: vscode.WorkspaceFoldersChangeEvent): Promise<void> {
        // Remove folders that were removed
        for (const folder of event.removed) {
            await this.removeFolder(folder);
        }

        // Add folders that were added
        for (const folder of event.added) {
            await this.addFolder(folder);
        }
    }

    /**
     * Handle document opened event
     */
    private async handleDocumentOpened(document: vscode.TextDocument): Promise<void> {
        // Auto-switch active folder based on opened document if configured
        if (this.workspaceConfig.autoSelectActiveProject && document.uri.scheme === 'file') {
            const folder = vscode.workspace.getWorkspaceFolder(document.uri);
            if (folder && this.folderToWorkspaceMap.has(folder) && folder !== this.activeFolder) {
                await this.setActiveFolder(folder);
            }
        }
    }

    /**
     * Handle document saved event
     */
    private async handleDocumentSaved(document: vscode.TextDocument): Promise<void> {
        // Refresh workspace if Cargo.toml was modified
        if (document.fileName.endsWith('Cargo.toml')) {
            const folder = vscode.workspace.getWorkspaceFolder(document.uri);
            if (folder) {
                const workspace = this.folderToWorkspaceMap.get(folder);
                if (workspace) {
                    await workspace.refreshTargets();
                    console.log(`Refreshed cargo workspace for ${folder.name} due to Cargo.toml change`);
                }
            }
        }
    }

    /**
     * Handle files renamed event
     */
    private async handleFilesRenamed(event: vscode.FileRenameEvent): Promise<void> {
        // Handle Cargo.toml files being renamed/moved
        for (const file of event.files) {
            if (file.oldUri.fsPath.endsWith('Cargo.toml') || file.newUri.fsPath.endsWith('Cargo.toml')) {
                const folder = vscode.workspace.getWorkspaceFolder(file.newUri);
                if (folder) {
                    const workspace = this.folderToWorkspaceMap.get(folder);
                    if (workspace) {
                        await workspace.refreshTargets();
                    }
                }
            }
        }
    }

    dispose(): void {
        this.subscriptions.forEach(sub => sub.dispose());
        this.folderToWorkspaceMap.clear();
    }
}
