import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTarget } from './cargoTarget';
import { StateManager } from './stateManager';
import { IconMapping } from './iconMapping';

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
    private stateManager?: StateManager;
    private groupByWorkspaceMember: boolean = true;
    private isRefreshing = false;
    private subscriptions: vscode.Disposable[] = [];

    // Filter state
    private workspaceMemberFilter: string = '';
    private targetTypeFilter: Set<string> = new Set(['bin', 'lib', 'example', 'bench']);
    private isTargetTypeFilterActive: boolean = false;
    private showFeatures: boolean = true;

    // Debounce timer for filter updates
    private filterUpdateTimer: NodeJS.Timeout | undefined;

    constructor() {
        // No configuration loading needed anymore
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

    /**
     * Set the state manager for persisting view state
     */
    setStateManager(stateManager: StateManager): void {
        this.stateManager = stateManager;
    }

    /**
     * Load filter and grouping state from persistence
     */
    async loadPersistedState(): Promise<void> {
        if (!this.stateManager) {
            return;
        }

        try {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                return;
            }

            const folderName = workspaceFolder.name;
            const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

            // Load persisted filter and grouping state
            this.groupByWorkspaceMember = this.stateManager.getGroupByWorkspaceMember(folderName, isMultiProject);
            this.workspaceMemberFilter = this.stateManager.getWorkspaceMemberFilter(folderName, isMultiProject);
            this.targetTypeFilter = new Set(this.stateManager.getTargetTypeFilter(folderName, isMultiProject));
            this.isTargetTypeFilterActive = this.stateManager.getIsTargetTypeFilterActive(folderName, isMultiProject);
            this.showFeatures = this.stateManager.getShowFeatures(folderName, isMultiProject);

            console.log('Loaded persisted outline view state for workspace:', folderName);
        } catch (error) {
            console.error('Failed to load persisted outline view state:', error);
        }
    }

    /**
     * Save current filter and grouping state to persistence
     */
    async saveCurrentState(): Promise<void> {
        if (!this.stateManager) {
            return;
        }

        try {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                return;
            }

            const folderName = workspaceFolder.name;
            const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

            // Save current filter and grouping state
            await this.stateManager.setGroupByWorkspaceMember(folderName, this.groupByWorkspaceMember, isMultiProject);
            await this.stateManager.setWorkspaceMemberFilter(folderName, this.workspaceMemberFilter, isMultiProject);
            await this.stateManager.setTargetTypeFilter(folderName, Array.from(this.targetTypeFilter), isMultiProject);
            await this.stateManager.setIsTargetTypeFilterActive(folderName, this.isTargetTypeFilterActive, isMultiProject);
            await this.stateManager.setShowFeatures(folderName, this.showFeatures, isMultiProject);
        } catch (error) {
            console.error('Failed to save outline view state:', error);
        }
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
        projectNode.iconPath = IconMapping.PROJECT;

        return [projectNode];
    }

    private getChildNodes(element: ProjectOutlineNode): ProjectOutlineNode[] {
        if (!this.workspace || !element.data) {
            return [];
        }

        switch (element.contextValue?.split(',')[0]) {
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

        // Add root-level Features node (only if features are enabled in filter)
        if (this.showFeatures) {
            if (this.workspace.isWorkspace) {
                // For workspace projects, show project-wide features
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
                rootFeaturesNode.iconPath = IconMapping.FEATURES_CONFIG;
                nodes.push(rootFeaturesNode);
            } else {
                // For single-crate projects, show package-specific features
                const packageName = this.workspace.projectName;
                const packageFeatures = this.workspace.getPackageFeatures(packageName);

                if (packageFeatures.length > 0) {
                    // Include 'all-features' as the first option, followed by specific features
                    const allFeatures = ['all-features', ...packageFeatures];

                    const featuresNode = new ProjectOutlineNode(
                        'Features',
                        vscode.TreeItemCollapsibleState.Expanded,
                        'features',
                        undefined,
                        undefined,
                        `${allFeatures.length} features`,
                        `Features available for package ${packageName}`,
                        { packageName: packageName, features: allFeatures }
                    );
                    featuresNode.iconPath = IconMapping.FEATURES_CONFIG;
                    nodes.push(featuresNode);
                }
            }
        }

        if (this.groupByWorkspaceMember && this.workspace.isWorkspace) {
            // Group by workspace member
            const workspaceMembers = this.workspace.getWorkspaceMembers();
            const filteredWorkspaceMembers = this.filterWorkspaceMembers(workspaceMembers);

            for (const [memberName, targets] of filteredWorkspaceMembers) {
                // Check if this package is selected
                const isSelectedPackage = this.workspace.selectedPackage === memberName;

                let label = memberName;
                // Add package selection indicator to the right of the label (CMake Tools pattern)
                if (isSelectedPackage) {
                    label += ' 📦'; // Package emoji for selected package
                }

                // Generate context value with selection state
                let contextValue = 'workspaceMember';
                if (isSelectedPackage) {
                    contextValue += ',isSelectedPackage';
                } else {
                    contextValue += ',canBeSelectedPackage';
                }

                const memberNode = new ProjectOutlineNode(
                    label,
                    vscode.TreeItemCollapsibleState.Expanded,
                    contextValue,
                    undefined,
                    undefined,
                    `${targets.length} targets`,
                    `Workspace member: ${memberName}`,
                    { memberName, targets }
                );

                // Always use the default package icon
                memberNode.iconPath = IconMapping.PACKAGE;

                nodes.push(memberNode);
            }
        } else {
            // Group by target type
            const allTargets = this.filterTargets(this.workspace.targets);
            const targetsByType = this.groupTargetsByType(allTargets);

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
                typeNode.iconPath = IconMapping.getIconForTargetType(type);
                nodes.push(typeNode);
            }
        }

        return nodes;
    }

    private createWorkspaceMemberChildren(data: { memberName: string; targets: CargoTarget[] }): ProjectOutlineNode[] {
        const nodes: ProjectOutlineNode[] = [];

        // Add Features node for this package (only if features are enabled in filter)
        if (this.workspace && this.showFeatures) {
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
                featuresNode.iconPath = IconMapping.FEATURES_CONFIG;
                nodes.push(featuresNode);
            }
        }

        // Add target groups
        const filteredTargets = this.filterTargets(data.targets);
        const targetsByType = this.groupTargetsByType(filteredTargets);

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
            typeNode.iconPath = IconMapping.getIconForTargetType(type);
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

            // Check selection states for right-side icon indicators
            let isBuildTarget = false;
            let isRunTarget = false;
            let isBenchTarget = false;

            if (this.workspace) {
                // For build targets, handle special case where "lib" is selected and target is a library
                const selectedBuildTarget = this.workspace.selectedBuildTarget;
                const selectedPackage = this.workspace.selectedPackage;

                if (selectedBuildTarget === 'lib' && target.kind.includes('lib')) {
                    // Only show icon if this library target belongs to the selected package
                    // If no package is selected, don't show library indicators
                    isBuildTarget = selectedPackage !== undefined && target.packageName === selectedPackage;
                } else {
                    isBuildTarget = selectedBuildTarget === target.name;
                }

                isRunTarget = this.workspace.selectedRunTarget === target.name;
                isBenchTarget = this.workspace.selectedBenchmarkTarget === target.name;
            }

            if (isDefault) {
                label += ' (default)';
            }

            // Add selection indicator icons to the right of the label (CMake Tools pattern)
            if (isBuildTarget) {
                label += ' 🔨'; // Hammer icon for build targets
            }
            if (isRunTarget) {
                label += ' 🚀'; // Rocket icon for run targets  
            }
            if (isBenchTarget) {
                label += ' ⚡'; // Lightning bolt icon for benchmark targets
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

            // Always use the default target type icon for the main iconPath
            targetNode.iconPath = IconMapping.getIconForTargetType(target.kind[0]);

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

            // For features that belong to the selected package, make them checkboxes
            const selectedPackage = this.workspace!.selectedPackage;
            const isFeatureInteractive = selectedPackage === data.packageName || (data.packageName === undefined && feature === 'all-features');

            // Add visual indicator for selected features
            const displayLabel = isSelected ? `✓ ${label}` : `  ${label}`;

            const featureNode = new ProjectOutlineNode(
                displayLabel,
                vscode.TreeItemCollapsibleState.None,
                isFeatureInteractive ? 'feature,interactive' : 'feature',
                undefined,
                isFeatureInteractive ? {
                    command: 'cargo-tools.projectOutline.toggleFeature',
                    title: 'Toggle Feature',
                    arguments: [feature, data.packageName]
                } : undefined,
                undefined,
                isSelected ? `Selected feature: ${feature}` : `Available feature: ${feature}`,
                { feature, packageName: data.packageName, isInteractive: isFeatureInteractive }
            );

            // Use appropriate icon for selection state
            featureNode.iconPath = isSelected ? IconMapping.SELECTED_STATE : IconMapping.UNSELECTED_STATE;

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

    private getContextValue(target: CargoTarget): string {
        const kinds = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];
        const contextParts = ['cargoTarget'];

        for (const kind of kinds) {
            switch (kind) {
                case 'bin':
                    contextParts.push('isExecutable', 'supportsBuild', 'supportsRun', 'supportsDebug');
                    break;
                case 'lib':
                    contextParts.push('isLibrary', 'supportsBuild');
                    break;
                case 'example':
                    contextParts.push('isExample', 'isExecutable', 'supportsBuild', 'supportsRun', 'supportsDebug');
                    break;
                case 'test':
                    contextParts.push('isTest', 'supportsBuild', 'supportsTest');
                    break;
                case 'bench':
                    contextParts.push('isBench', 'supportsBuild', 'supportsBench');
                    break;
            }
        }

        // Add selection state information
        if (this.workspace) {
            const selectedBuildTarget = this.workspace.selectedBuildTarget;
            const selectedRunTarget = this.workspace.selectedRunTarget;
            const selectedBenchmarkTarget = this.workspace.selectedBenchmarkTarget;

            // For build targets, handle library vs other targets differently
            const isSelectedBuildTarget = target.kind.includes('lib')
                ? selectedBuildTarget === 'lib'
                : selectedBuildTarget === target.name;

            if (isSelectedBuildTarget) {
                contextParts.push('isSelectedBuildTarget');
            } else if (contextParts.includes('supportsBuild')) {
                contextParts.push('canBeSelectedBuildTarget');
            }

            if (selectedRunTarget === target.name) {
                contextParts.push('isSelectedRunTarget');
            } else if (contextParts.includes('supportsRun')) {
                contextParts.push('canBeSelectedRunTarget');
            }

            if (selectedBenchmarkTarget === target.name) {
                contextParts.push('isSelectedBenchmarkTarget');
            } else if (contextParts.includes('supportsBench')) {
                contextParts.push('canBeSelectedBenchmarkTarget');
            }
        }

        return contextParts.join(',');
    }

    private getTooltip(target: CargoTarget): string {
        const kinds = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];
        const kindStr = kinds.join(', ');
        return `${target.name} (${kindStr})\nPackage: ${target.packageName}\nPath: ${target.srcPath}`;
    }

    // Filter methods
    public async setWorkspaceMemberFilter(): Promise<void> {
        if (!this.workspace || !this.workspace.isWorkspace) {
            // Fallback to simple input for non-workspace projects
            const input = await vscode.window.showInputBox({
                prompt: 'Enter workspace member filter (leave empty to clear)',
                value: this.workspaceMemberFilter,
                placeHolder: 'Filter workspace members...'
            });

            if (input !== undefined) {
                this.workspaceMemberFilter = input.trim();
                this.refresh();
            }
            return;
        }

        // Get all workspace members for preview
        const workspaceMembers = this.workspace.getWorkspaceMembers();
        const allMemberNames = Array.from(workspaceMembers.keys()).sort();

        // Store original filter value to restore on cancel
        const originalFilter = this.workspaceMemberFilter;
        let wasAccepted = false;

        // Create QuickPick for real-time filtering with preview
        const quickPick = vscode.window.createQuickPick();
        quickPick.placeholder = 'Type to filter workspace members, then press Enter to apply...';
        quickPick.value = this.workspaceMemberFilter;
        quickPick.matchOnDescription = true;
        quickPick.matchOnDetail = true;

        // Function to update QuickPick items based on current filter
        const updateItems = (filterValue: string) => {
            const filter = filterValue.toLowerCase().trim();

            if (!filter) {
                // Show all members when no filter
                const memberItems = allMemberNames.map(memberName => ({
                    label: `$(package) ${memberName}`,
                    description: `${workspaceMembers.get(memberName)?.length || 0} targets`
                }));

                quickPick.items = memberItems;
            } else {
                // Filter and show matching members
                const matchingMembers = allMemberNames.filter(name =>
                    name.toLowerCase().includes(filter)
                );

                const memberItems = matchingMembers.map(memberName => ({
                    label: `$(package) ${memberName}`,
                    description: `${workspaceMembers.get(memberName)?.length || 0} targets`
                }));

                quickPick.items = memberItems;
            }
        };

        // Initial population
        updateItems(quickPick.value);

        // Ensure no default selection and keep clearing selections
        quickPick.selectedItems = [];

        // Real-time update as user types with debouncing
        const disposable = quickPick.onDidChangeValue((value) => {
            // Clear existing timer
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }

            // Set a new timer for debounced UI update
            this.filterUpdateTimer = setTimeout(() => {
                updateItems(value);
                // Clear any selections after updating items
                quickPick.selectedItems = [];
            }, 100); // Fast response for UI updates            // Also update the actual filter in real-time for immediate tree preview
            // Use a separate shorter debounce for tree updates
            setTimeout(() => {
                this.workspaceMemberFilter = value.trim();
                this.refresh();
            }, 200); // Slightly longer to avoid too frequent tree refreshes
        });

        quickPick.onDidAccept(() => {
            // Apply the typed filter value (items are unselectable)
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }
            wasAccepted = true;

            // Always use the typed value as filter since items are unselectable
            this.workspaceMemberFilter = quickPick.value.trim();

            this.refresh();
            this.saveCurrentState(); // Persist filter changes
            quickPick.hide();
        });

        quickPick.onDidHide(() => {
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }

            // If user canceled (didn't accept), restore original filter
            if (!wasAccepted) {
                this.workspaceMemberFilter = originalFilter;
                this.refresh();
            }

            disposable.dispose();
            quickPick.dispose();
        });

        quickPick.show();
    }

    public async showTargetTypeFilter(): Promise<void> {
        interface FilterQuickPickItem extends vscode.QuickPickItem {
            targetType: string;
            isFeature: boolean;
        }

        const allTargetTypes = ['bin', 'lib', 'example', 'bench'];

        // Store original filter values to restore on cancel
        const originalTargetTypeFilter = new Set(this.targetTypeFilter);
        const originalShowFeatures = this.showFeatures;
        const originalIsTargetTypeFilterActive = this.isTargetTypeFilterActive;
        let wasAccepted = false;

        const allFilterOptions: FilterQuickPickItem[] = [
            ...allTargetTypes.map(type => ({
                label: this.getDisplayNameForTargetType(type),
                picked: this.targetTypeFilter.has(type),
                targetType: type,
                isFeature: false
            })),
            {
                label: 'Features',
                picked: this.showFeatures,
                targetType: 'features',
                isFeature: true
            }
        ];

        // Use QuickPick for real-time updates
        const quickPick = vscode.window.createQuickPick<FilterQuickPickItem>();
        quickPick.placeholder = 'Select what to show in Project Outline';
        quickPick.canSelectMany = true;
        quickPick.items = allFilterOptions;
        quickPick.selectedItems = allFilterOptions.filter(item => item.picked);

        // Update filter in real-time as user changes selection
        const disposable = quickPick.onDidChangeSelection((items) => {
            // Clear existing timer
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }

            // Set a new timer for debounced update
            this.filterUpdateTimer = setTimeout(() => {
                this.targetTypeFilter.clear();
                this.showFeatures = false;

                for (const item of items) {
                    if (item.isFeature) {
                        this.showFeatures = true;
                    } else {
                        this.targetTypeFilter.add(item.targetType);
                    }
                }

                this.isTargetTypeFilterActive = this.targetTypeFilter.size < allTargetTypes.length || !this.showFeatures;
                this.refresh();
            }, 100); // Shorter debounce for real-time feel
        });

        quickPick.onDidAccept(() => {
            // Immediate update on accept
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }
            wasAccepted = true;
            this.saveCurrentState(); // Persist filter changes
            quickPick.hide();
        });

        quickPick.onDidHide(() => {
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }

            // If user canceled (didn't accept), restore original filter values
            if (!wasAccepted) {
                this.targetTypeFilter = originalTargetTypeFilter;
                this.showFeatures = originalShowFeatures;
                this.isTargetTypeFilterActive = originalIsTargetTypeFilterActive;
                this.refresh();
            }

            disposable.dispose();
            quickPick.dispose();
        });

        quickPick.show();
    }

    public clearWorkspaceMemberFilter(): void {
        this.workspaceMemberFilter = '';
        this.refresh();
        this.saveCurrentState(); // Persist filter changes
    }

    public clearTargetTypeFilter(): void {
        this.targetTypeFilter = new Set(['bin', 'lib', 'example', 'bench']);
        this.showFeatures = true;
        this.isTargetTypeFilterActive = false;
        this.refresh();
        this.saveCurrentState(); // Persist filter changes
    }

    public clearAllFilters(): void {
        this.workspaceMemberFilter = '';
        this.targetTypeFilter = new Set(['bin', 'lib', 'example', 'bench']);
        this.showFeatures = true;
        this.isTargetTypeFilterActive = false;
        this.refresh();
        this.saveCurrentState(); // Persist filter changes
    }

    public toggleWorkspaceMemberGrouping(): void {
        this.groupByWorkspaceMember = !this.groupByWorkspaceMember;
        this.refresh();
        this.saveCurrentState(); // Persist grouping changes
    }

    // Apply filters to workspace members
    private filterWorkspaceMembers(workspaceMembers: Map<string, CargoTarget[]>): Map<string, CargoTarget[]> {
        const filtered = new Map<string, CargoTarget[]>();

        for (const [memberName, targets] of workspaceMembers) {
            // Apply workspace member filter
            if (this.workspaceMemberFilter && !memberName.toLowerCase().includes(this.workspaceMemberFilter.toLowerCase())) {
                continue;
            }

            // Apply target type filter
            const filteredTargets = targets.filter(target => {
                const targetKinds = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];
                return targetKinds.some(kind => this.targetTypeFilter.has(kind));
            });

            if (filteredTargets.length > 0) {
                filtered.set(memberName, filteredTargets);
            }
        }

        return filtered;
    }

    // Apply filters to targets
    private filterTargets(targets: CargoTarget[]): CargoTarget[] {
        return targets.filter(target => {
            // Apply workspace member filter
            if (this.workspaceMemberFilter) {
                // If packageName is undefined, filter it out when workspace member filter is active
                if (!target.packageName) {
                    return false;
                }
                // Filter out if packageName doesn't contain the filter string
                if (!target.packageName.toLowerCase().includes(this.workspaceMemberFilter.toLowerCase())) {
                    return false;
                }
            }

            // Apply target type filter
            const targetKinds = Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'];
            return targetKinds.some(kind => this.targetTypeFilter.has(kind));
        });
    }

    // Check if any filters are active
    public hasActiveFilters(): boolean {
        return this.workspaceMemberFilter !== '' || this.isTargetTypeFilterActive;
    }

    dispose(): void {
        if (this.filterUpdateTimer) {
            clearTimeout(this.filterUpdateTimer);
        }
        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];
    }
}
