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
    private subscriptions: vscode.Disposable[] = [];

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
        // Dispose existing subscriptions
        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];

        this.workspace = workspace;

        // Set up new subscriptions if workspace is available
        if (workspace) {
            this.subscriptions.push(
                workspace.onDidChangeSelectedPackage(() => this.refresh()),
                workspace.onDidChangeSelectedBuildTarget(() => this.refresh()),
                workspace.onDidChangeSelectedRunTarget(() => this.refresh()),
                workspace.onDidChangeSelectedBenchmarkTarget(() => this.refresh()),
                workspace.onDidChangeSelectedFeatures(() => this.refresh()),
                workspace.onDidChangeTargets(() => this.refresh())
            );
        }

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

        // Single root node: Project name
        const projectNode = new ProjectOutlineNode(
            this.workspace.projectName,
            vscode.TreeItemCollapsibleState.Expanded,
            'project',
            undefined,
            undefined,
            `${this.workspace.targets.length} targets`,
            `Rust project: ${this.workspace.projectName}`,
            { projectName: this.workspace.projectName }
        );
        projectNode.iconPath = new vscode.ThemeIcon('symbol-package');

        return [projectNode];
    }

    private getChildNodes(element: ProjectOutlineNode): ProjectOutlineNode[] {
        if (!this.workspace || !element.data) {
            return [];
        }

        switch (element.contextValue) {
            case 'project':
                return this.createProjectChildren();
            case 'workspaceMember':
                return this.createWorkspaceMemberChildren(element.data);
            case 'targetType':
                return this.createTargetTypeChildren(element.data);
            case 'targetTypeGroup':
                return this.createTargetNodes(element.data.targets);
            case 'features':
                return this.createFeatureNodes(element.data);
            default:
                return [];
        }
    }

    private createProjectChildren(): ProjectOutlineNode[] {
        if (!this.workspace) {
            return [];
        }

        const nodes: ProjectOutlineNode[] = [];

        // Add root-level Features node 
        const rootFeaturesNode = new ProjectOutlineNode(
            'Features',
            vscode.TreeItemCollapsibleState.Expanded,
            'features',
            undefined,
            undefined,
            'Project features',
            'Features available for the entire project',
            { packageName: undefined, features: ['all-features'] }
        );
        rootFeaturesNode.iconPath = new vscode.ThemeIcon('settings-gear');
        nodes.push(rootFeaturesNode);

        if (this.groupByWorkspaceMember && this.workspace.isWorkspace) {
            // Group by workspace member
            const workspaceMembers = this.workspace.getWorkspaceMembers();

            for (const [memberName, targets] of workspaceMembers) {
                // Check if this package is selected
                const isSelectedPackage = this.workspace.selectedPackage === memberName;

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

                // Use icon to indicate selection state
                if (isSelectedPackage) {
                    memberNode.iconPath = new vscode.ThemeIcon('package', new vscode.ThemeColor('list.activeSelectionForeground'));
                } else {
                    memberNode.iconPath = new vscode.ThemeIcon('package');
                }

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

    private createWorkspaceMemberChildren(data: { memberName: string; targets: CargoTarget[] }): ProjectOutlineNode[] {
        const nodes: ProjectOutlineNode[] = [];

        // Add Features node for this package
        if (this.workspace) {
            const packageFeatures = this.workspace.getPackageFeatures(data.memberName);
            if (packageFeatures.length > 0) {
                const featuresNode = new ProjectOutlineNode(
                    'Features',
                    vscode.TreeItemCollapsibleState.Expanded,
                    'features',
                    undefined,
                    undefined,
                    `${packageFeatures.length} features`,
                    `Features available for package ${data.memberName}`,
                    { packageName: data.memberName, features: packageFeatures }
                );
                featuresNode.iconPath = new vscode.ThemeIcon('settings-gear');
                nodes.push(featuresNode);
            }
        }

        // Add target groups
        const targetsByType = this.groupTargetsByType(data.targets);

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
            let label = target.name;

            // Check selection states for icon styling
            let isBuildTarget = false;
            let isRunTarget = false;
            let isBenchTarget = false;

            if (this.workspace) {
                isBuildTarget = this.workspace.selectedBuildTarget === target.name;
                isRunTarget = this.workspace.selectedRunTarget === target.name;
                isBenchTarget = this.workspace.selectedBenchmarkTarget === target.name;
            }

            if (isDefault) {
                label += ' (default)';
            }

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

            // Set icon based on target type and selection state
            if (isBuildTarget) {
                // Use build icon (similar to CMake Tools)
                targetNode.iconPath = new vscode.ThemeIcon('tools', new vscode.ThemeColor('list.activeSelectionForeground'));
            } else if (isRunTarget) {
                // Use run/play icon (similar to CMake Tools)
                targetNode.iconPath = new vscode.ThemeIcon('play', new vscode.ThemeColor('list.activeSelectionForeground'));
            } else if (isBenchTarget) {
                // Use benchmark/stopwatch icon 
                targetNode.iconPath = new vscode.ThemeIcon('pulse', new vscode.ThemeColor('list.activeSelectionForeground'));
            } else if (isDefault) {
                // Use star for default target
                targetNode.iconPath = new vscode.ThemeIcon('star', new vscode.ThemeColor('list.highlightForeground'));
            } else {
                // Use default target type icon
                targetNode.iconPath = this.getIconForTarget(target);
            }

            return targetNode;
        });
    }

    private createFeatureNodes(data: { packageName: string | undefined; features: string[] }): ProjectOutlineNode[] {
        if (!this.workspace) {
            return [];
        }

        return data.features.map(feature => {
            const selectedFeatures = this.workspace!.selectedFeatures;
            const isSelected = selectedFeatures.has(feature);
            const label = feature === 'all-features' ? 'All features' : feature;

            // Add visual indicator for selected features
            const displayLabel = isSelected ? `✓ ${label}` : `  ${label}`;

            const featureNode = new ProjectOutlineNode(
                displayLabel,
                vscode.TreeItemCollapsibleState.None,
                'feature',
                undefined,
                undefined,
                undefined,
                isSelected ? `Selected feature: ${feature}` : `Available feature: ${feature}`,
                { feature, packageName: data.packageName }
            );

            // Use appropriate icon for selection state
            featureNode.iconPath = new vscode.ThemeIcon(isSelected ? 'check' : 'circle-outline');

            return featureNode;
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

    dispose(): void {
        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];
    }
}
