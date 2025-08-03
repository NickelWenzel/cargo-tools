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
        this.workspace = workspace;
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

        // Workspace node
        const workspaceNode = new ProjectStatusNode(
            'Workspace',
            vscode.TreeItemCollapsibleState.Expanded,
            'workspace',
            undefined,
            this.workspace?.workspaceRoot ? vscode.workspace.asRelativePath(this.workspace.workspaceRoot) : undefined,
            this.workspace?.workspaceRoot
        );
        workspaceNode.iconPath = new vscode.ThemeIcon('folder');
        nodes.push(workspaceNode);

        // Build Configuration node
        const configNode = new ProjectStatusNode(
            'Build Configuration',
            vscode.TreeItemCollapsibleState.Expanded,
            'buildConfiguration'
        );
        configNode.iconPath = new vscode.ThemeIcon('settings-gear');
        nodes.push(configNode);

        // Actions node
        const actionsNode = new ProjectStatusNode(
            'Actions',
            vscode.TreeItemCollapsibleState.Expanded,
            'actions'
        );
        actionsNode.iconPath = new vscode.ThemeIcon('play');
        nodes.push(actionsNode);

        return nodes;
    }

    private getChildNodes(element: ProjectStatusNode): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        switch (element.contextValue) {
            case 'workspace':
                return this.createWorkspaceChildren();
            case 'buildConfiguration':
                return this.createBuildConfigurationChildren();
            case 'actions':
                return this.createActionsChildren();
            default:
                return [];
        }
    }

    private createWorkspaceChildren(): ProjectStatusNode[] {
        if (!this.workspace) {
            return [];
        }

        const nodes: ProjectStatusNode[] = [];

        // Project node
        const projectName = this.workspace.manifest?.package?.name ||
            this.workspace.workspaceMembers[0] ||
            'Unknown Project';
        const cargoTomlPath = `${this.workspace.workspaceRoot}/Cargo.toml`;
        const projectNode = new ProjectStatusNode(
            projectName,
            vscode.TreeItemCollapsibleState.None,
            'project',
            {
                command: 'vscode.open',
                title: 'Open Cargo.toml',
                arguments: [vscode.Uri.file(cargoTomlPath)]
            },
            'Active Project',
            'Click to open Cargo.toml'
        );
        projectNode.iconPath = new vscode.ThemeIcon('package');
        nodes.push(projectNode);

        return nodes;
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

        // Default target node
        const defaultTarget = this.workspace.currentTarget;
        const targetNode = new ProjectStatusNode(
            defaultTarget?.name || '[No Target Selected]',
            vscode.TreeItemCollapsibleState.None,
            'defaultTarget',
            {
                command: 'cargo-tools.selectTarget',
                title: 'Change Default Target'
            },
            'Default Target',
            'Click to change default target'
        );
        targetNode.iconPath = new vscode.ThemeIcon('target');
        nodes.push(targetNode);

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
