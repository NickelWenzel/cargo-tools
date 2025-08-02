import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTarget } from './cargoTarget';

export class TargetTreeItem extends vscode.TreeItem {
    constructor(
        public readonly target: CargoTarget,
        public readonly isActive: boolean
    ) {
        super(target.displayName, vscode.TreeItemCollapsibleState.None);
        
        this.tooltip = `Target: ${target.name}\nKind: ${target.kind.join(', ')}\nPath: ${target.srcPath}`;
        this.contextValue = 'cargoTarget';
        this.command = {
            command: 'cargo-tools.selectTarget',
            title: 'Select Target',
            arguments: [target]
        };

        // Set icon based on target type
        if (target.isExecutable) {
            this.iconPath = new vscode.ThemeIcon('play');
        } else if (target.isLibrary) {
            this.iconPath = new vscode.ThemeIcon('package');
        } else if (target.isTest) {
            this.iconPath = new vscode.ThemeIcon('beaker');
        } else if (target.isBench) {
            this.iconPath = new vscode.ThemeIcon('graph');
        } else if (target.isExample) {
            this.iconPath = new vscode.ThemeIcon('book');
        } else {
            this.iconPath = new vscode.ThemeIcon('file');
        }

        if (isActive) {
            this.description = '(active)';
        }
    }
}

export class TargetsTreeProvider implements vscode.TreeDataProvider<TargetTreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<TargetTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private workspace: CargoWorkspace) {
        workspace.onDidChangeTarget(() => {
            this._onDidChangeTreeData.fire();
        });

        workspace.onDidChangeTargets(() => {
            this._onDidChangeTreeData.fire();
        });
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TargetTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: TargetTreeItem): Thenable<TargetTreeItem[]> {
        if (!element) {
            // Root level - return all targets
            const targets = this.workspace.targets;
            const currentTarget = this.workspace.currentTarget;
            
            return Promise.resolve(
                targets.map(target => new TargetTreeItem(target, target === currentTarget))
            );
        }
        
        return Promise.resolve([]);
    }
}
