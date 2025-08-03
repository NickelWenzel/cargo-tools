import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';

export class ProjectStatusNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue?: string,
        public readonly command?: vscode.Command,
        public readonly description?: string,
        public readonly tooltip?: string
    ) {
        super(label, collapsibleState);
        this.contextValue = contextValue;
        this.command = command;
        this.description = description;
        this.tooltip = tooltip;
    }
}

export class ProjectStatusTreeProvider implements vscode.TreeDataProvider<ProjectStatusNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<ProjectStatusNode | undefined | null | void> = new vscode.EventEmitter<ProjectStatusNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ProjectStatusNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private workspace?: CargoWorkspace;
    private isRefreshing = false;
    private subscriptions: vscode.Disposable[] = [];

    constructor() { }

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
        for (const subscription of this.subscriptions) {
            subscription.dispose();
        }
        this.subscriptions = [];

        this.workspace = workspace;

        // Set up new subscriptions
        if (this.workspace) {
            this.subscriptions.push(
                this.workspace.onDidChangeProfile(() => this.refresh()),
                this.workspace.onDidChangeSelectedPackage(() => this.refresh()),
                this.workspace.onDidChangeTarget(() => this.refresh()),
                this.workspace.onDidChangeTargets(() => this.refresh())
            );
        }

        this.refresh();
    }

    getTreeItem(element: ProjectStatusNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: ProjectStatusNode): Promise<ProjectStatusNode[]> {
        if (!this.workspace) {
            return [new ProjectStatusNode(
                'No Cargo workspace found',
                vscode.TreeItemCollapsibleState.None,
                'noWorkspace'
            )];
        }

        if (!element) {
            // Root level - show main categories
            return this.createRootNodes();
        }

        return this.getChildNodes(element);
    }

    private createRootNodes(): ProjectStatusNode[] {
        const nodes: ProjectStatusNode[] = [];

        // Build Configuration node
        const configNode = new ProjectStatusNode(
            'Build Configuration',
            vscode.TreeItemCollapsibleState.Expanded,
            'buildConfiguration'
        );
        configNode.iconPath = new vscode.ThemeIcon('settings-gear');
        nodes.push(configNode);

        // Package Selection node
        const packageNode = new ProjectStatusNode(
            'Package Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'packageSelection'
        );
        packageNode.iconPath = new vscode.ThemeIcon('package');
        nodes.push(packageNode);

        // Build Target Selection node
        const targetNode = new ProjectStatusNode(
            'Build Target Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'targetSelection'
        );
        targetNode.iconPath = new vscode.ThemeIcon('target');
        nodes.push(targetNode);

        return nodes;
    }

    private getChildNodes(element: ProjectStatusNode): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        switch (element.contextValue) {
            case 'buildConfiguration':
                return this.createBuildConfigurationChildren();
            case 'packageSelection':
                return this.createPackageSelectionChildren();
            case 'targetSelection':
                return this.createTargetSelectionChildren();
            default:
                return [];
        }
    }

    private createBuildConfigurationChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const nodes: ProjectStatusNode[] = [];

        // Profile node
        const currentProfile = this.workspace.currentProfile || CargoProfile.dev;
        const profileNode = new ProjectStatusNode(
            currentProfile,
            vscode.TreeItemCollapsibleState.None,
            'profile',
            {
                command: 'cargo-tools.selectProfile',
                title: 'Change Build Profile'
            },
            'Build Profile',
            'Click to change build profile'
        );
        profileNode.iconPath = new vscode.ThemeIcon('settings');
        nodes.push(profileNode);

        return nodes;
    }

    private createPackageSelectionChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const nodes: ProjectStatusNode[] = [];

        if (this.workspace.isWorkspace) {
            // Multi-package workspace - show current selection with dropdown
            const selectedPackage = this.workspace.selectedPackage;
            const displayName = selectedPackage || 'All packages';

            const packageNode = new ProjectStatusNode(
                displayName,
                vscode.TreeItemCollapsibleState.None,
                'package',
                {
                    command: 'cargo-tools.selectPackage',
                    title: 'Change Package Selection'
                },
                'Package Selection',
                'Click to change package selection'
            );
            packageNode.iconPath = new vscode.ThemeIcon('package');
            nodes.push(packageNode);
        } else {
            // Single package - read-only display
            const defaultNode = new ProjectStatusNode(
                'default',
                vscode.TreeItemCollapsibleState.None,
                'package-default',
                undefined,
                'Single package',
                'Single package project'
            );
            defaultNode.iconPath = new vscode.ThemeIcon('package');
            nodes.push(defaultNode);
        }

        return nodes;
    }

    private createTargetSelectionChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const nodes: ProjectStatusNode[] = [];

        // Add "All" option
        const allNode = new ProjectStatusNode(
            'All',
            vscode.TreeItemCollapsibleState.None,
            'target-all',
            undefined,
            'All targets',
            'Build all targets (no target specification)'
        );
        allNode.iconPath = new vscode.ThemeIcon('target');
        nodes.push(allNode);

        // Group targets by type
        const targetsByType = new Map<string, string[]>();

        for (const target of this.workspace.targets) {
            for (const kind of target.kind) {
                if (!targetsByType.has(kind)) {
                    targetsByType.set(kind, []);
                }
                targetsByType.get(kind)!.push(target.name);
            }
        }

        // Add library if exists
        if (targetsByType.has('lib')) {
            const libNode = new ProjectStatusNode(
                'lib',
                vscode.TreeItemCollapsibleState.None,
                'target-lib',
                undefined,
                'Library target',
                'Build library (--lib)'
            );
            libNode.iconPath = new vscode.ThemeIcon('library');
            nodes.push(libNode);
        }

        // Add binaries group
        if (targetsByType.has('bin')) {
            const binTargets = targetsByType.get('bin')!;
            if (binTargets.length === 1) {
                const binNode = new ProjectStatusNode(
                    `bin: ${binTargets[0]}`,
                    vscode.TreeItemCollapsibleState.None,
                    'target-bin',
                    undefined,
                    'Binary target',
                    `Build binary: ${binTargets[0]}`
                );
                binNode.iconPath = new vscode.ThemeIcon('file-binary');
                nodes.push(binNode);
            } else {
                const binsNode = new ProjectStatusNode(
                    'bins',
                    vscode.TreeItemCollapsibleState.Collapsed,
                    'target-bins-group',
                    undefined,
                    `${binTargets.length} binaries`,
                    'Binary targets group'
                );
                binsNode.iconPath = new vscode.ThemeIcon('file-binary');
                nodes.push(binsNode);
            }
        }

        // Add examples group
        if (targetsByType.has('example')) {
            const exampleTargets = targetsByType.get('example')!;
            const examplesNode = new ProjectStatusNode(
                'examples',
                vscode.TreeItemCollapsibleState.Collapsed,
                'target-examples-group',
                undefined,
                `${exampleTargets.length} examples`,
                'Example targets group'
            );
            examplesNode.iconPath = new vscode.ThemeIcon('file-code');
            nodes.push(examplesNode);
        }

        // Add benchmarks group
        if (targetsByType.has('bench')) {
            const benchTargets = targetsByType.get('bench')!;
            const benchmarksNode = new ProjectStatusNode(
                'benchmarks',
                vscode.TreeItemCollapsibleState.Collapsed,
                'target-benchmarks-group',
                undefined,
                `${benchTargets.length} benchmarks`,
                'Benchmark targets group'
            );
            benchmarksNode.iconPath = new vscode.ThemeIcon('dashboard');
            nodes.push(benchmarksNode);
        }

        return nodes;
    }

    private createActionsChildren(): ProjectStatusNode[] {
        const nodes: ProjectStatusNode[] = [];

        // Build action
        const buildNode = new ProjectStatusNode(
            'Build',
            vscode.TreeItemCollapsibleState.None,
            'action-build',
            {
                command: 'cargo-tools.executeBuildAction',
                title: 'Build'
            },
            undefined,
            'Build the current target'
        );
        buildNode.iconPath = new vscode.ThemeIcon('tools');
        nodes.push(buildNode);

        // Run action
        const runNode = new ProjectStatusNode(
            'Run',
            vscode.TreeItemCollapsibleState.None,
            'action-run',
            {
                command: 'cargo-tools.executeRunAction',
                title: 'Run'
            },
            undefined,
            'Run the current target'
        );
        runNode.iconPath = new vscode.ThemeIcon('play');
        nodes.push(runNode);

        // Test action
        const testNode = new ProjectStatusNode(
            'Test',
            vscode.TreeItemCollapsibleState.None,
            'action-test',
            {
                command: 'cargo-tools.executeTestAction',
                title: 'Test'
            },
            undefined,
            'Run tests'
        );
        testNode.iconPath = new vscode.ThemeIcon('beaker');
        nodes.push(testNode);

        // Bench action  
        const benchNode = new ProjectStatusNode(
            'Bench',
            vscode.TreeItemCollapsibleState.None,
            'action-bench',
            {
                command: 'cargo-tools.executeBenchAction',
                title: 'Bench'
            },
            undefined,
            'Run benchmarks'
        );
        benchNode.iconPath = new vscode.ThemeIcon('dashboard');
        nodes.push(benchNode);

        return nodes;
    }
}
