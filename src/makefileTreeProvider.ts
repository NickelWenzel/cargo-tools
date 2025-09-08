import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';

export class MakefileNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue?: string,
        public readonly resourceUri?: vscode.Uri,
        public readonly command?: vscode.Command,
        public readonly description?: string,
        public readonly tooltip?: string,
        public readonly data?: any
    ) {
        super(label, collapsibleState);
        this.contextValue = contextValue;
        this.resourceUri = resourceUri;
        this.command = command;
        this.description = description;
        this.tooltip = tooltip;
    }
}

export class MakefileTreeProvider implements vscode.TreeDataProvider<MakefileNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<MakefileNode | undefined | null | void> = new vscode.EventEmitter<MakefileNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<MakefileNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private workspace?: CargoWorkspace;
    private subscriptions: vscode.Disposable[] = [];

    constructor() {
        // Initialize the tree provider
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    updateWorkspace(workspace: CargoWorkspace | undefined): void {
        // Dispose existing subscriptions
        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];

        this.workspace = workspace;

        // Set up new subscriptions if workspace is available
        if (workspace) {
            this.subscriptions.push(
                workspace.onDidChangeTargets(() => this.refresh())
            );
        }

        this.refresh();
    }

    getTreeItem(element: MakefileNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: MakefileNode): Promise<MakefileNode[]> {
        if (!this.workspace || !this.workspace.hasMakefileToml) {
            return [new MakefileNode(
                'No Makefile.toml found',
                vscode.TreeItemCollapsibleState.None,
                'noMakefile'
            )];
        }

        if (!element) {
            // Root level - for now return empty as requested
            return [];
        }

        return [];
    }

    dispose(): void {
        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];
    }
}
