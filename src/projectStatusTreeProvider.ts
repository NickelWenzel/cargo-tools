import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';
import { CargoTarget } from './cargoTarget';

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
                this.workspace.onDidChangeTargets(() => this.refresh()),
                this.workspace.onDidChangeSelectedBuildTarget(() => this.refresh()),
                this.workspace.onDidChangeSelectedRunTarget(() => this.refresh()),
                this.workspace.onDidChangeSelectedBenchmarkTarget(() => this.refresh()),
                this.workspace.onDidChangeSelectedFeatures(() => this.refresh())
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

        // Target Selection node (parent for all target types)
        const targetNode = new ProjectStatusNode(
            'Target Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'targetSelection'
        );
        targetNode.iconPath = new vscode.ThemeIcon('target');
        nodes.push(targetNode);

        // Feature Selection node
        const featureNode = new ProjectStatusNode(
            'Feature Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'featureSelection'
        );
        featureNode.iconPath = new vscode.ThemeIcon('symbol-misc');
        nodes.push(featureNode);

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
            case 'buildTargetSelection':
                return this.createBuildTargetSelectionChildren();
            case 'runTargetSelection':
                return this.createRunTargetSelectionChildren();
            case 'benchmarkTargetSelection':
                return this.createBenchmarkTargetSelectionChildren();
            case 'featureSelection':
                return this.createFeatureSelectionChildren();
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
            const displayName = selectedPackage || 'No selection';

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

        // Build Target Selection
        const buildTargetNode = new ProjectStatusNode(
            'Build Target Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'buildTargetSelection',
            undefined,
            'Build target selection',
            'Select which target to build'
        );
        buildTargetNode.iconPath = new vscode.ThemeIcon('tools');
        nodes.push(buildTargetNode);

        // Run Target Selection
        const runTargetNode = new ProjectStatusNode(
            'Run Target Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'runTargetSelection',
            undefined,
            'Run target selection',
            'Select which target to run'
        );
        runTargetNode.iconPath = new vscode.ThemeIcon('play');
        nodes.push(runTargetNode);

        // Benchmark Target Selection
        const benchmarkTargetNode = new ProjectStatusNode(
            'Benchmark Target Selection',
            vscode.TreeItemCollapsibleState.Expanded,
            'benchmarkTargetSelection',
            undefined,
            'Benchmark target selection',
            'Select which benchmark to run'
        );
        benchmarkTargetNode.iconPath = new vscode.ThemeIcon('dashboard');
        nodes.push(benchmarkTargetNode);

        return nodes;
    }


    private createBuildTargetSelectionChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const selectedBuildTarget = this.workspace.selectedBuildTarget;

        if (selectedBuildTarget) {
            // Show selected target with command to change selection
            const node = new ProjectStatusNode(
                selectedBuildTarget,
                vscode.TreeItemCollapsibleState.None,
                'selected-build-target',
                {
                    command: 'cargo-tools.selectBuildTarget',
                    title: 'Change Build Target'
                },
                `Selected build target: ${selectedBuildTarget}`,
                `Click to change build target`
            );
            node.iconPath = new vscode.ThemeIcon('check');
            return [node];
        } else {
            // No build target selected - always show "No selection"
            const node = new ProjectStatusNode(
                'No selection',
                vscode.TreeItemCollapsibleState.None,
                'default-build-target',
                {
                    command: 'cargo-tools.selectBuildTarget',
                    title: 'Select Build Target'
                },
                'No build target selected (build all targets)',
                'Click to select specific build target'
            );
            node.iconPath = new vscode.ThemeIcon('target');
            return [node];
        }
    }

    private createRunTargetSelectionChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const selectedRunTarget = this.workspace.selectedRunTarget;

        if (selectedRunTarget) {
            // Show selected target with command to change selection
            const node = new ProjectStatusNode(
                selectedRunTarget,
                vscode.TreeItemCollapsibleState.None,
                'selected-run-target',
                {
                    command: 'cargo-tools.selectRunTarget',
                    title: 'Change Run Target'
                },
                `Selected run target: ${selectedRunTarget}`,
                `Click to change run target`
            );
            node.iconPath = new vscode.ThemeIcon('check');
            return [node];
        } else {
            // Show disabled or default state
            const selectedPackage = this.workspace.selectedPackage;
            if (!selectedPackage) {
                // No package selected - disabled
                const node = new ProjectStatusNode(
                    'Disabled when no package selected',
                    vscode.TreeItemCollapsibleState.None,
                    'disabled-run-target',
                    undefined,
                    'Select a specific package to run targets',
                    'Run targets require a specific package selection'
                );
                node.iconPath = new vscode.ThemeIcon('circle-slash');
                return [node];
            } else {
                // Specific package selected - check if runnable targets exist
                const packageTargets = this.getTargetsForPackage(selectedPackage);
                const targetsByType = this.groupTargetsByType(packageTargets);
                const hasRunnableTargets = targetsByType.has('bin') || targetsByType.has('example');

                if (hasRunnableTargets) {
                    // Show default "Auto-detect" option
                    const node = new ProjectStatusNode(
                        'Auto-detect',
                        vscode.TreeItemCollapsibleState.None,
                        'default-run-target',
                        {
                            command: 'cargo-tools.selectRunTarget',
                            title: 'Select Run Target'
                        },
                        'Auto-detect run target (default)',
                        'Click to select specific run target'
                    );
                    node.iconPath = new vscode.ThemeIcon('play');
                    return [node];
                } else {
                    // No runnable targets in package
                    const node = new ProjectStatusNode(
                        'No runnable targets in package',
                        vscode.TreeItemCollapsibleState.None,
                        'no-run-targets',
                        undefined,
                        'No binaries or examples to run',
                        'This package has no runnable targets'
                    );
                    node.iconPath = new vscode.ThemeIcon('circle-slash');
                    return [node];
                }
            }
        }
    }

    private createBenchmarkTargetSelectionChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const selectedBenchmarkTarget = this.workspace.selectedBenchmarkTarget;

        if (selectedBenchmarkTarget) {
            // Show selected target with command to change selection
            const node = new ProjectStatusNode(
                selectedBenchmarkTarget,
                vscode.TreeItemCollapsibleState.None,
                'selected-benchmark-target',
                {
                    command: 'cargo-tools.selectBenchmarkTarget',
                    title: 'Change Benchmark Target'
                },
                `Selected benchmark target: ${selectedBenchmarkTarget}`,
                `Click to change benchmark target`
            );
            node.iconPath = new vscode.ThemeIcon('check');
            return [node];
        } else {
            // Show "No selection" when no specific benchmark target is selected
            const node = new ProjectStatusNode(
                'No selection',
                vscode.TreeItemCollapsibleState.None,
                'default-benchmark-target',
                {
                    command: 'cargo-tools.selectBenchmarkTarget',
                    title: 'Select Benchmark Target'
                },
                'No benchmark target selected',
                'Click to select benchmark target'
            );
            node.iconPath = new vscode.ThemeIcon('dashboard');
            return [node];
        }
    }

    private createFeatureSelectionChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const availableFeatures = this.workspace.getAvailableFeatures();
        const selectedFeatures = this.workspace.selectedFeatures;
        const nodes: ProjectStatusNode[] = [];

        // Add feature toggle options
        for (const feature of availableFeatures) {
            const isSelected = selectedFeatures.has(feature);
            const label = feature === 'all-features' ? 'All features' : feature;

            const node = new ProjectStatusNode(
                label,
                vscode.TreeItemCollapsibleState.None,
                'feature-item',
                {
                    command: 'cargo-tools.toggleFeature',
                    title: `Toggle ${feature}`,
                    arguments: [feature]
                },
                isSelected ? `✓ ${label}` : `  ${label}`,
                `Click to ${isSelected ? 'unselect' : 'select'} ${feature}`
            );

            // Use checkbox icons to indicate selection state
            node.iconPath = new vscode.ThemeIcon(isSelected ? 'check' : 'circle-outline');
            nodes.push(node);
        }

        return nodes;
    }

    private getTargetsForPackage(packageName: string): CargoTarget[] {
        if (!this.workspace) {
            return [];
        }

        return this.workspace.targets.filter(target => target.packageName === packageName);
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

        return groups;
    }
}