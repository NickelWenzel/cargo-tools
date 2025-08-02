import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import * as path from 'path';
import * as fs from 'fs';

export class WorkspaceTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly itemType: 'workspace' | 'package' | 'file' | 'dependency',
        public readonly resourceUri?: vscode.Uri,
        public readonly description?: string
    ) {
        super(label, collapsibleState);

        this.contextValue = itemType;
        this.tooltip = this.resourceUri ? this.resourceUri.fsPath : this.label;

        // Set icons based on item type
        switch (itemType) {
            case 'workspace':
                this.iconPath = new vscode.ThemeIcon('folder-library');
                break;
            case 'package':
                this.iconPath = new vscode.ThemeIcon('package');
                break;
            case 'file':
                if (this.label.endsWith('.rs')) {
                    this.iconPath = new vscode.ThemeIcon('file-code');
                } else if (this.label === 'Cargo.toml') {
                    this.iconPath = new vscode.ThemeIcon('settings-gear');
                } else {
                    this.iconPath = new vscode.ThemeIcon('file');
                }
                break;
            case 'dependency':
                this.iconPath = new vscode.ThemeIcon('extensions');
                break;
        }

        // Make files clickable
        if (itemType === 'file' && this.resourceUri) {
            this.command = {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [this.resourceUri]
            };
        }

        if (description) {
            this.description = description;
        }
    }
}

export class WorkspaceTreeProvider implements vscode.TreeDataProvider<WorkspaceTreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<WorkspaceTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private workspace: CargoWorkspace) {
        // Subscribe to workspace events to automatically refresh
        workspace.onDidChangeTargets(() => {
            this._onDidChangeTreeData.fire();
        });
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: WorkspaceTreeItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: WorkspaceTreeItem): Promise<WorkspaceTreeItem[]> {
        if (!element) {
            // Root level - show workspace info
            const items: WorkspaceTreeItem[] = [];

            // Show workspace name
            const workspaceName = path.basename(this.workspace.workspaceRoot);
            items.push(new WorkspaceTreeItem(
                workspaceName,
                vscode.TreeItemCollapsibleState.Expanded,
                'workspace',
                undefined,
                'Cargo Workspace'
            ));

            return items;
        }

        if (element.itemType === 'workspace') {
            return this.getWorkspaceChildren();
        }

        return [];
    }

    private async getWorkspaceChildren(): Promise<WorkspaceTreeItem[]> {
        const items: WorkspaceTreeItem[] = [];

        // Add Cargo.toml
        const cargoTomlPath = path.join(this.workspace.workspaceRoot, 'Cargo.toml');
        if (fs.existsSync(cargoTomlPath)) {
            items.push(new WorkspaceTreeItem(
                'Cargo.toml',
                vscode.TreeItemCollapsibleState.None,
                'file',
                vscode.Uri.file(cargoTomlPath)
            ));
        }

        // Add source files
        const srcDir = path.join(this.workspace.workspaceRoot, 'src');
        if (fs.existsSync(srcDir)) {
            const srcFiles = await this.getSourceFiles(srcDir);
            items.push(...srcFiles);
        }

        // Add targets
        if (this.workspace.targets.length > 0) {
            const targetsItem = new WorkspaceTreeItem(
                'Targets',
                vscode.TreeItemCollapsibleState.Expanded,
                'package'
            );
            items.push(targetsItem);
        }

        // Add dependencies if manifest is available
        if (this.workspace.manifest?.dependencies) {
            const depsItem = new WorkspaceTreeItem(
                'Dependencies',
                vscode.TreeItemCollapsibleState.Collapsed,
                'package'
            );
            items.push(depsItem);
        }

        return items;
    }

    private async getSourceFiles(dir: string): Promise<WorkspaceTreeItem[]> {
        const items: WorkspaceTreeItem[] = [];

        try {
            const files = await fs.promises.readdir(dir);

            for (const file of files) {
                const filePath = path.join(dir, file);
                const stat = await fs.promises.stat(filePath);

                if (stat.isFile() && file.endsWith('.rs')) {
                    items.push(new WorkspaceTreeItem(
                        file,
                        vscode.TreeItemCollapsibleState.None,
                        'file',
                        vscode.Uri.file(filePath)
                    ));
                }
            }
        } catch (error) {
            console.error('Error reading source files:', error);
        }

        return items;
    }
}
