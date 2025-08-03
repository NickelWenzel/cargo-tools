import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTarget, TargetActionType } from './cargoTarget';

export class TargetTreeItem extends vscode.TreeItem {
    constructor(
        public readonly target: CargoTarget,
        public readonly isActive: boolean
    ) {
        super(target.displayName, vscode.TreeItemCollapsibleState.None);

        this.tooltip = `Target: ${target.name}\nKind: ${target.kind.join(', ')}\nPath: ${target.srcPath}`;

        // Build context value with target capabilities for context menu
        // Build context value for menu contributions
        const contextParts = ['cargoTarget'];

        // Add traditional type-based contexts for backward compatibility
        if (target.isExecutable) {
            contextParts.push('cargoTarget.isExecutable');
        }
        if (target.isTest) {
            contextParts.push('cargoTarget.isTest');
        }
        if (target.isBench) {
            contextParts.push('cargoTarget.isBench');
        }
        if (target.isExample) {
            contextParts.push('cargoTarget.isExample');
        }

        // Add action-based contexts for new features
        const supportedActions = target.supportedActionTypes;
        for (const action of supportedActions) {
            contextParts.push(`cargoTarget.supports${action.charAt(0).toUpperCase() + action.slice(1)}`);
        }

        this.contextValue = contextParts.join(' && ');

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

export class WorkspaceMemberItem extends vscode.TreeItem {
    constructor(
        public readonly memberName: string,
        public readonly targets: CargoTarget[]
    ) {
        super(memberName, vscode.TreeItemCollapsibleState.Expanded);
        this.contextValue = 'workspaceMember';
        this.iconPath = new vscode.ThemeIcon('package');
        this.tooltip = `Workspace member: ${memberName} (${targets.length} targets)`;
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

export class TargetsTreeProvider implements vscode.TreeDataProvider<TargetTreeItem | TargetGroupItem | WorkspaceMemberItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<TargetTreeItem | TargetGroupItem | WorkspaceMemberItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private workspace: CargoWorkspace) {
        workspace.onDidChangeTarget(() => {
            this._onDidChangeTreeData.fire();
        });

        workspace.onDidChangeTargets(() => {
            this._onDidChangeTreeData.fire();
        });

        // Listen for configuration changes to refresh the tree when grouping setting changes
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('cargoTools.groupTargetsByWorkspaceMember')) {
                this._onDidChangeTreeData.fire();
            }
        });
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TargetTreeItem | TargetGroupItem | WorkspaceMemberItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: TargetTreeItem | TargetGroupItem | WorkspaceMemberItem): Thenable<(TargetTreeItem | TargetGroupItem | WorkspaceMemberItem)[]> {
        if (!element) {
            // Root level - determine if this is a workspace or single package
            return this.getRootChildren();
        } else if (element instanceof WorkspaceMemberItem) {
            // Show target groups for this workspace member
            const groups = this.groupTargetsByKind(element.targets);
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

    private async getRootChildren(): Promise<(TargetTreeItem | TargetGroupItem | WorkspaceMemberItem)[]> {
        const targets = this.workspace.targets;
        const config = vscode.workspace.getConfiguration('cargoTools');
        const groupByWorkspaceMember = config.get<boolean>('groupTargetsByWorkspaceMember', true);

        // Check if this is a workspace with multiple members and grouping is enabled
        const workspaceMembers = this.workspace.getWorkspaceMembers();

        if (groupByWorkspaceMember && workspaceMembers.size > 1) {
            // Multi-member workspace: group by workspace member first
            return Array.from(workspaceMembers.entries()).map(([memberName, memberTargets]) =>
                new WorkspaceMemberItem(memberName, memberTargets)
            );
        } else {
            // Single package or grouping disabled: group directly by kind (legacy behavior)
            const groups = this.groupTargetsByKind(targets);
            return Array.from(groups.entries()).map(([kind, targets]) =>
                new TargetGroupItem(kind, targets)
            );
        }
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
