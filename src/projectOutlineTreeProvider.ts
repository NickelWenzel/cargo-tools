import * as vscode from 'vscode';
import * as path from 'path';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTarget } from './cargoTarget';

export class ProjectOutlineNode extends vscode.TreeItem {
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

export class ProjectOutlineTreeProvider implements vscode.TreeDataProvider<ProjectOutlineNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<ProjectOutlineNode | undefined | null | void> = new vscode.EventEmitter<ProjectOutlineNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ProjectOutlineNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private workspace?: CargoWorkspace;
    private groupByWorkspaceMember: boolean = true;
    private isRefreshing = false;

    constructor() {
        // Load configuration
        this.updateConfiguration();

        // Listen for configuration changes
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('cargoTools.groupTargetsByWorkspaceMember')) {
                this.updateConfiguration();
                this.refresh();
            }
        });
    }

    private updateConfiguration(): void {
        this.groupByWorkspaceMember = vscode.workspace.getConfiguration('cargoTools').get('groupTargetsByWorkspaceMember', true);
    }

    refresh(): void {
        if (!this.isRefreshing) {
            this.isRefreshing = true;
            this._onDidChangeTreeData.fire();
            // Reset flag after a short delay to prevent rapid refreshes
            setTimeout(() => {
                this.isRefreshing = false;
            }, 100);
        }
    }

    updateWorkspace(workspace: CargoWorkspace | undefined): void {
        this.workspace = workspace;
        this.refresh();
    }

    getTreeItem(element: ProjectOutlineNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: ProjectOutlineNode): Promise<ProjectOutlineNode[]> {
        if (!this.workspace) {
            return [new ProjectOutlineNode(
                'No Cargo workspace found',
                vscode.TreeItemCollapsibleState.None,
                'noWorkspace'
            )];
        }

        if (!element) {
            // Root level
            return this.createRootNodes();
        }

        return this.getChildNodes(element);
    }

    private createRootNodes(): ProjectOutlineNode[] {
        if (!this.workspace) {
            return [];
        }

        const nodes: ProjectOutlineNode[] = [];

        if (this.groupByWorkspaceMember && this.workspace.isWorkspace) {
            // Group by workspace member
            const workspaceMembers = this.workspace.getWorkspaceMembers();

            for (const [memberName, targets] of workspaceMembers) {
                const memberNode = new ProjectOutlineNode(
                    memberName,
                    vscode.TreeItemCollapsibleState.Expanded,
                    'workspaceMember',
                    undefined,
                    undefined,
                    `${targets.length} targets`,
                    `Workspace member: ${memberName}`,
                    { memberName, targets }
                );
                memberNode.iconPath = new vscode.ThemeIcon('package');
                nodes.push(memberNode);
            }
        } else {
            // Group by target type
            const targetsByType = this.groupTargetsByType(this.workspace.targets);

            for (const [type, targets] of targetsByType) {
                const typeNode = new ProjectOutlineNode(
                    this.getDisplayNameForTargetType(type),
                    vscode.TreeItemCollapsibleState.Expanded,
                    'targetType',
                    undefined,
                    undefined,
                    `${targets.length} ${type}${targets.length === 1 ? '' : 's'}`,
                    `Target type: ${type}`,
                    { type, targets }
                );
                typeNode.iconPath = this.getIconForTargetType(type);
                nodes.push(typeNode);
            }
        }

        return nodes;
    }

    private getChildNodes(element: ProjectOutlineNode): ProjectOutlineNode[] {
        if (!this.workspace || !element.data) {
            return [];
        }

        switch (element.contextValue) {
            case 'workspaceMember':
                return this.createWorkspaceMemberChildren(element.data);
            case 'targetType':
                return this.createTargetTypeChildren(element.data);
            case 'targetTypeGroup':
                return this.createTargetNodes(element.data.targets);
            default:
                return [];
        }
    }

    private createWorkspaceMemberChildren(data: { memberName: string; targets: CargoTarget[] }): ProjectOutlineNode[] {
        const targetsByType = this.groupTargetsByType(data.targets);
        const nodes: ProjectOutlineNode[] = [];

        for (const [type, targets] of targetsByType) {
            const typeNode = new ProjectOutlineNode(
                this.getDisplayNameForTargetType(type),
                vscode.TreeItemCollapsibleState.Expanded,
                'targetTypeGroup',
                undefined,
                undefined,
                `${targets.length} ${type}${targets.length === 1 ? '' : 's'}`,
                `Target type: ${type}`,
                { type, targets }
            );
            typeNode.iconPath = this.getIconForTargetType(type);
            nodes.push(typeNode);
        }

        return nodes;
    }

    private createTargetTypeChildren(data: { type: string; targets: CargoTarget[] }): ProjectOutlineNode[] {
        return this.createTargetNodes(data.targets);
    }

    private createTargetNodes(targets: CargoTarget[]): ProjectOutlineNode[] {
        return targets.map(target => {
            const isDefault = this.workspace?.currentTarget === target;
            const label = isDefault ? `${target.name} (default)` : target.name;

            const targetNode = new ProjectOutlineNode(
                label,
                vscode.TreeItemCollapsibleState.None,
                this.getContextValue(target),
                vscode.Uri.file(target.srcPath),
                {
                    command: 'vscode.open',
                    title: 'Open Source File',
                    arguments: [vscode.Uri.file(target.srcPath)]
                },
                target.packageName !== target.name ? target.packageName : undefined,
                this.getTooltip(target),
                target
            );

            targetNode.iconPath = this.getIconForTarget(target);

            if (isDefault) {
                targetNode.iconPath = new vscode.ThemeIcon('star', new vscode.ThemeColor('list.highlightForeground'));
            }

            return targetNode;
        });
    }

    private groupTargetsByType(targets: CargoTarget[]): Map<string, CargoTarget[]> {
        const groups = new Map<string, CargoTarget[]>();

        for (const target of targets) {
            const types = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];

            for (const type of types) {
                if (!groups.has(type)) {
                    groups.set(type, []);
                }
                groups.get(type)!.push(target);
            }
        }

        // Sort groups by priority: bin, lib, example, test, bench, others
        const sortedGroups = new Map<string, CargoTarget[]>();
        const priority = ['bin', 'lib', 'example', 'test', 'bench'];

        for (const type of priority) {
            if (groups.has(type)) {
                sortedGroups.set(type, groups.get(type)!);
                groups.delete(type);
            }
        }

        // Add remaining types
        for (const [type, targets] of groups) {
            sortedGroups.set(type, targets);
        }

        return sortedGroups;
    }

    private getDisplayNameForTargetType(type: string): string {
        switch (type) {
            case 'bin':
                return 'Binaries';
            case 'lib':
                return 'Libraries';
            case 'example':
                return 'Examples';
            case 'test':
                return 'Tests';
            case 'bench':
                return 'Benchmarks';
            default:
                return type.charAt(0).toUpperCase() + type.slice(1);
        }
    }

    private getIconForTargetType(type: string): vscode.ThemeIcon {
        switch (type) {
            case 'bin':
                return new vscode.ThemeIcon('file-code');
            case 'lib':
                return new vscode.ThemeIcon('library');
            case 'example':
                return new vscode.ThemeIcon('lightbulb');
            case 'test':
                return new vscode.ThemeIcon('beaker');
            case 'bench':
                return new vscode.ThemeIcon('dashboard');
            default:
                return new vscode.ThemeIcon('file');
        }
    }

    private getIconForTarget(target: CargoTarget): vscode.ThemeIcon {
        if (!target.kind || !Array.isArray(target.kind) || target.kind.length === 0) {
            return new vscode.ThemeIcon('file');
        }

        // Use the first kind for icon selection
        const primaryKind = target.kind[0];
        return this.getIconForTargetType(primaryKind);
    }

    private getContextValue(target: CargoTarget): string {
        const kinds = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];
        const contextParts = ['cargoTarget'];

        for (const kind of kinds) {
            switch (kind) {
                case 'bin':
                    contextParts.push('isExecutable', 'supportsBuild', 'supportsRun');
                    break;
                case 'lib':
                    contextParts.push('isLibrary', 'supportsBuild');
                    break;
                case 'example':
                    contextParts.push('isExample', 'isExecutable', 'supportsBuild', 'supportsRun');
                    break;
                case 'test':
                    contextParts.push('isTest', 'supportsBuild', 'supportsTest');
                    break;
                case 'bench':
                    contextParts.push('isBench', 'supportsBuild', 'supportsBench');
                    break;
            }
        }

        return contextParts.join(',');
    }

    private getTooltip(target: CargoTarget): string {
        const kinds = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];
        const kindStr = kinds.join(', ');
        return `${target.name} (${kindStr})\nPackage: ${target.packageName}\nPath: ${target.srcPath}`;
    }
}
