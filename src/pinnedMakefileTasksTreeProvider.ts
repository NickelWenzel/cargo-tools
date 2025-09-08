import * as vscode from 'vscode';
import { CargoWorkspace, CargoMakeTask } from './cargoWorkspace';
import { StateManager } from './stateManager';

export class PinnedTaskNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue?: string,
        public readonly resourceUri?: vscode.Uri,
        public readonly command?: vscode.Command,
        public readonly description?: string,
        public readonly tooltip?: string,
        public readonly taskName?: string
    ) {
        super(label, collapsibleState);
        this.contextValue = contextValue;
        this.resourceUri = resourceUri;
        this.command = command;
        this.description = description;
        this.tooltip = tooltip;
    }
}

export class PinnedMakefileTasksTreeProvider implements vscode.TreeDataProvider<PinnedTaskNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<PinnedTaskNode | undefined | null | void> = new vscode.EventEmitter<PinnedTaskNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<PinnedTaskNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private workspace?: CargoWorkspace;
    private stateManager?: StateManager;
    private pinnedTasks: string[] = [];

    constructor() {
        // Initialize the tree provider
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    setStateManager(stateManager: StateManager): void {
        this.stateManager = stateManager;
    }

    loadPersistedState(): void {
        if (this.stateManager) {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                return;
            }

            const folderName = workspaceFolder.name;
            const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

            this.pinnedTasks = this.stateManager.getPinnedMakefileTasks(folderName, isMultiProject);
            this.refresh();
        }
    }

    updateWorkspace(workspace: CargoWorkspace | undefined): void {
        this.workspace = workspace;

        // Do not load state automatically here - only after state manager is set
        this.refresh();
    }

    getTreeItem(element: PinnedTaskNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: PinnedTaskNode): Promise<PinnedTaskNode[]> {
        if (!this.workspace || !this.workspace.hasMakefileToml) {
            return [new PinnedTaskNode(
                'No Makefile.toml found',
                vscode.TreeItemCollapsibleState.None,
                'no-makefile',
                undefined,
                undefined,
                'No Makefile.toml found in workspace',
                'No Makefile.toml found in workspace'
            )];
        }

        if (element) {
            // No children for task nodes
            return [];
        }

        // Root level - show pinned tasks
        if (this.pinnedTasks.length === 0) {
            return [new PinnedTaskNode(
                'No pinned tasks',
                vscode.TreeItemCollapsibleState.None,
                'empty-pinned-list',
                undefined,
                undefined,
                'Click "Add" to pin a task',
                'Click "Add" to pin a task'
            )];
        }

        const allTasks = this.workspace.makeTasks;
        const taskMap = new Map<string, CargoMakeTask>();
        for (const task of allTasks) {
            taskMap.set(task.name, task);
        }

        const pinnedTaskNodes: PinnedTaskNode[] = [];

        for (const taskName of this.pinnedTasks) {
            const task = taskMap.get(taskName);
            if (task) {
                const node = new PinnedTaskNode(
                    task.name,
                    vscode.TreeItemCollapsibleState.None,
                    'pinned-task',
                    undefined,
                    undefined,
                    task.description || 'No description',
                    `${task.name}: ${task.description || 'No description'}`,
                    task.name
                );

                // Add gear icon (consistent with Makefile view)
                node.iconPath = new vscode.ThemeIcon('gear');

                pinnedTaskNodes.push(node);
            } else {
                // Task no longer exists, show it as invalid
                const node = new PinnedTaskNode(
                    `${taskName} (not found)`,
                    vscode.TreeItemCollapsibleState.None,
                    'pinned-task-invalid',
                    undefined,
                    undefined,
                    'Task not found in Makefile.toml',
                    `Task "${taskName}" not found in Makefile.toml`,
                    taskName
                );

                // Add warning icon
                node.iconPath = new vscode.ThemeIcon('warning');

                pinnedTaskNodes.push(node);
            }
        }

        return pinnedTaskNodes;
    }

    async addPinnedTask(taskName: string): Promise<void> {
        if (!this.stateManager) {
            return;
        }

        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            return;
        }

        const folderName = workspaceFolder.name;
        const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

        // Check if task is already pinned
        if (this.pinnedTasks.includes(taskName)) {
            vscode.window.showInformationMessage(`Task "${taskName}" is already pinned`);
            return;
        }

        // Add to pinned tasks
        this.pinnedTasks.push(taskName);
        await this.stateManager.setPinnedMakefileTasks(folderName, this.pinnedTasks, isMultiProject);

        this.refresh();
        vscode.window.showInformationMessage(`Added "${taskName}" to pinned tasks`);
    }

    async removePinnedTask(taskName: string): Promise<void> {
        if (!this.stateManager) {
            return;
        }

        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            return;
        }

        const folderName = workspaceFolder.name;
        const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

        // Remove from pinned tasks
        const index = this.pinnedTasks.indexOf(taskName);
        if (index !== -1) {
            this.pinnedTasks.splice(index, 1);
            await this.stateManager.setPinnedMakefileTasks(folderName, this.pinnedTasks, isMultiProject);

            this.refresh();
            vscode.window.showInformationMessage(`Removed "${taskName}" from pinned tasks`);
        }
    }

    async showAddTaskQuickPick(): Promise<void> {
        if (!this.workspace || !this.workspace.hasMakefileToml) {
            vscode.window.showErrorMessage('No Makefile.toml found in workspace');
            return;
        }

        const allTasks = this.workspace.makeTasks;

        // Filter out already pinned tasks
        const availableTasks = allTasks.filter((task: CargoMakeTask) => !this.pinnedTasks.includes(task.name));

        if (availableTasks.length === 0) {
            vscode.window.showInformationMessage('All tasks are already pinned');
            return;
        }

        const quickPickItems: vscode.QuickPickItem[] = availableTasks.map((task: CargoMakeTask) => ({
            label: task.name,
            description: task.description || 'No description',
            detail: task.category ? `Category: ${task.category}` : undefined
        }));

        const selected = await vscode.window.showQuickPick(quickPickItems, {
            placeHolder: 'Select a task to pin',
            matchOnDescription: true,
            matchOnDetail: true
        });

        if (selected) {
            await this.addPinnedTask(selected.label);
        }
    }

    getPinnedTasks(): string[] {
        return [...this.pinnedTasks];
    }
}
