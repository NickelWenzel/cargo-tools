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

        // Set icon based on target type following CMake Tools patterns
        if (target.isExecutable) {
            this.iconPath = new vscode.ThemeIcon('debug-start');
        } else if (target.isLibrary) {
            this.iconPath = new vscode.ThemeIcon('library');
        } else if (target.isTest) {
            this.iconPath = new vscode.ThemeIcon('beaker');
        } else if (target.isBench) {
            this.iconPath = new vscode.ThemeIcon('dashboard');
        } else if (target.isExample) {
            this.iconPath = new vscode.ThemeIcon('lightbulb');
        } else {
            this.iconPath = new vscode.ThemeIcon('file');
        }

        if (isActive) {
            this.description = '(active)';
        }
    }
}

export class TargetGroupItem extends vscode.TreeItem {
    constructor(
        public readonly kind: string,
        public readonly targets: CargoTarget[]
    ) {
        super(`${kind} (${targets.length})`, vscode.TreeItemCollapsibleState.Expanded);
        this.contextValue = 'targetGroup';
        this.iconPath = new vscode.ThemeIcon('folder');
        this.tooltip = `${targets.length} ${kind} target${targets.length !== 1 ? 's' : ''}`;
    }
}

export class TargetsTreeProvider implements vscode.TreeDataProvider<TargetTreeItem | TargetGroupItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<TargetTreeItem | TargetGroupItem | undefined | null | void>();
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

    getTreeItem(element: TargetTreeItem | TargetGroupItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: TargetTreeItem | TargetGroupItem): Thenable<(TargetTreeItem | TargetGroupItem)[]> {
        if (!element) {
            // Root level - group targets by kind
            const targets = this.workspace.targets;
            const groups = this.groupTargetsByKind(targets);

            return Promise.resolve(
                Array.from(groups.entries()).map(([kind, targets]) =>
                    new TargetGroupItem(kind, targets)
                )
            );
        } else if (element instanceof TargetGroupItem) {
            // Show targets in this group
            const currentTarget = this.workspace.currentTarget;
            return Promise.resolve(
                element.targets.map(target => new TargetTreeItem(target, target === currentTarget))
            );
        }

        return Promise.resolve([]);
    }

    private groupTargetsByKind(targets: CargoTarget[]): Map<string, CargoTarget[]> {
        const groups = new Map<string, CargoTarget[]>();

        for (const target of targets) {
            // Use the primary kind (first in the array)
            const primaryKind = target.kind[0] || 'unknown';

            if (!groups.has(primaryKind)) {
                groups.set(primaryKind, []);
            }
            groups.get(primaryKind)!.push(target);
        }

        // Sort groups by priority (bin first, then lib, etc.)
        const sortedGroups = new Map<string, CargoTarget[]>();
        const kindOrder = ['bin', 'lib', 'example', 'test', 'bench'];

        for (const kind of kindOrder) {
            if (groups.has(kind)) {
                sortedGroups.set(kind, groups.get(kind)!);
            }
        }

        // Add any remaining kinds
        for (const [kind, targets] of groups) {
            if (!sortedGroups.has(kind)) {
                sortedGroups.set(kind, targets);
            }
        }

        return sortedGroups;
    }
}
