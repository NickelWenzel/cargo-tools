import * as vscode from 'vscode';
import { CargoWorkspace, CargoMakeTask } from './cargoWorkspace';

export class MakefileNode extends vscode.TreeItem {
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

export class MakefileTreeProvider implements vscode.TreeDataProvider<MakefileNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<MakefileNode | undefined | null | void> = new vscode.EventEmitter<MakefileNode | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<MakefileNode | undefined | null | void> = this._onDidChangeTreeData.event;

    private workspace?: CargoWorkspace;
    private subscriptions: vscode.Disposable[] = [];

    // Filter state
    private taskFilter: string = '';
    private categoryFilter: Set<string> = new Set();
    private isCategoryFilterActive: boolean = false;
    private filterUpdateTimer?: NodeJS.Timeout;

    constructor() {
        // Initialize the tree provider
    }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    updateWorkspace(workspace: CargoWorkspace | undefined): void {
        // Dispose existing subscriptions
        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];

        this.workspace = workspace;

        // Set up new subscriptions if workspace is available
        if (workspace) {
            this.subscriptions.push(
                workspace.onDidChangeTargets(() => this.refresh())
            );
        }

        this.refresh();
    }

    getTreeItem(element: MakefileNode): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: MakefileNode): Promise<MakefileNode[]> {
        if (!this.workspace || !this.workspace.hasMakefileToml) {
            return [new MakefileNode(
                'No Makefile.toml found',
                vscode.TreeItemCollapsibleState.None,
                'noMakefile'
            )];
        }

        if (!element) {
            // Root level - show categories
            return this.createCategoryNodes();
        }

        // If element is a category, show tasks in that category
        if (element.contextValue === 'category') {
            return this.createTaskNodes(element.data.category);
        }

        return [];
    }

    private createCategoryNodes(): MakefileNode[] {
        if (!this.workspace) {
            return [];
        }

        const tasks = this.workspace.makeTasks;

        // Initialize category filter with all categories if not set
        if (this.categoryFilter.size === 0 && !this.isCategoryFilterActive) {
            const allCategories = [...new Set(tasks.map(task => task.category))];
            this.categoryFilter = new Set(allCategories);
        }

        const categories = new Map<string, CargoMakeTask[]>();

        // Apply task filter if active
        const filteredTasks = this.taskFilter.trim()
            ? tasks.filter(task =>
                task.name.toLowerCase().includes(this.taskFilter.toLowerCase())
            )
            : tasks;

        // Group filtered tasks by category
        for (const task of filteredTasks) {
            if (!categories.has(task.category)) {
                categories.set(task.category, []);
            }
            categories.get(task.category)!.push(task);
        }

        // Apply category filter - only show categories that are in the filter
        const categoryNodes: MakefileNode[] = [];
        for (const [categoryName, categoryTasks] of categories) {
            // Skip categories that are filtered out
            if (this.isCategoryFilterActive && !this.categoryFilter.has(categoryName)) {
                continue;
            }

            const categoryNode = new MakefileNode(
                categoryName,
                vscode.TreeItemCollapsibleState.Expanded,
                'category',
                undefined,
                undefined,
                `${categoryTasks.length} tasks`,
                `Category: ${categoryName}`,
                { category: categoryName, tasks: categoryTasks }
            );
            categoryNode.iconPath = new vscode.ThemeIcon('folder');
            categoryNodes.push(categoryNode);
        }

        // Sort categories alphabetically
        return categoryNodes.sort((a, b) => a.label!.toString().localeCompare(b.label!.toString()));
    }

    private createTaskNodes(category: string): MakefileNode[] {
        if (!this.workspace) {
            return [];
        }

        let tasks = this.workspace.makeTasks.filter(task => task.category === category);

        // Apply task filter if active
        if (this.taskFilter.trim()) {
            tasks = tasks.filter(task =>
                task.name.toLowerCase().includes(this.taskFilter.toLowerCase())
            );
        }

        return tasks.map(task => {
            const taskNode = new MakefileNode(
                task.name,
                vscode.TreeItemCollapsibleState.None,
                'task',
                undefined,
                {
                    command: 'cargo-tools.makefile.runTask',
                    title: 'Run Task',
                    arguments: [task.name]
                },
                task.description,
                `Run task: ${task.name}`,
                { task }
            );
            taskNode.iconPath = new vscode.ThemeIcon('play');
            return taskNode;
        });
    }

    // Filter methods
    public async setTaskFilter(): Promise<void> {
        if (!this.workspace || !this.workspace.hasMakefileToml) {
            vscode.window.showWarningMessage('No Makefile.toml found in workspace');
            return;
        }

        // Get all tasks for preview
        const allTasks = this.workspace.makeTasks;
        const allTaskNames = allTasks.map(task => task.name).sort();

        // Store original filter value to restore on cancel
        const originalFilter = this.taskFilter;
        let wasAccepted = false;

        // Create QuickPick for real-time filtering with preview
        const quickPick = vscode.window.createQuickPick();
        quickPick.placeholder = 'Type to filter tasks by name, then press Enter to apply...';
        quickPick.value = this.taskFilter;
        quickPick.matchOnDescription = true;
        quickPick.matchOnDetail = true;

        // Function to update QuickPick items based on current filter
        const updateItems = (filterValue: string) => {
            const filter = filterValue.toLowerCase().trim();

            if (!filter) {
                // Show all task names when no filter
                const taskItems = allTaskNames.map(taskName => {
                    const task = allTasks.find(t => t.name === taskName);
                    return {
                        label: taskName,
                        description: task?.description || '',
                        detail: `Category: ${task?.category}`
                    };
                });

                quickPick.items = taskItems;
            } else {
                // Filter and show only matching task names
                const matchingTasks = allTasks.filter(task =>
                    task.name.toLowerCase().includes(filter)
                );

                const taskItems = matchingTasks.map(task => ({
                    label: task.name,
                    description: task.description,
                    detail: `Category: ${task.category}`
                }));

                quickPick.items = taskItems;
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
            }, 100); // Fast response for UI updates

            // Also update the actual filter in real-time for immediate tree preview
            // Use a separate shorter debounce for tree updates
            setTimeout(() => {
                this.taskFilter = value.trim();
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
            this.taskFilter = quickPick.value.trim();
            this.refresh();

            quickPick.hide();
        });

        quickPick.onDidHide(() => {
            // Clean up timer
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }

            // If user cancelled (did not accept), restore original filter
            if (!wasAccepted) {
                this.taskFilter = originalFilter;
                this.refresh();
            }

            disposable.dispose();
            quickPick.dispose();
        });

        quickPick.show();
    }

    public editTaskFilter(): void {
        this.setTaskFilter();
    }

    public clearTaskFilter(): void {
        this.taskFilter = '';
        this.refresh();
    }

    public get currentTaskFilter(): string {
        return this.taskFilter;
    }

    // Category filter methods
    public async showCategoryFilter(): Promise<void> {
        if (!this.workspace || !this.workspace.hasMakefileToml) {
            vscode.window.showWarningMessage('No Makefile.toml found in workspace');
            return;
        }

        interface CategoryFilterQuickPickItem extends vscode.QuickPickItem {
            categoryName: string;
        }

        // Get all available categories
        const allTasks = this.workspace.makeTasks;
        const allCategories = [...new Set(allTasks.map(task => task.category))].sort();

        // Initialize filter if not set
        if (this.categoryFilter.size === 0 && !this.isCategoryFilterActive) {
            this.categoryFilter = new Set(allCategories);
        }

        // Store original filter values to restore on cancel
        const originalCategoryFilter = new Set(this.categoryFilter);
        const originalIsCategoryFilterActive = this.isCategoryFilterActive;
        let wasAccepted = false;

        const allFilterOptions: CategoryFilterQuickPickItem[] = allCategories.map(category => ({
            label: category,
            description: `${allTasks.filter(t => t.category === category).length} tasks`,
            picked: this.categoryFilter.has(category),
            categoryName: category
        }));

        // Use QuickPick for real-time updates
        const quickPick = vscode.window.createQuickPick<CategoryFilterQuickPickItem>();
        quickPick.placeholder = 'Select which categories to show in Makefile view';
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
                this.categoryFilter.clear();

                for (const item of items) {
                    this.categoryFilter.add(item.categoryName);
                }

                this.isCategoryFilterActive = this.categoryFilter.size < allCategories.length;
                this.refresh();
            }, 100); // Shorter debounce for real-time feel
        });

        quickPick.onDidAccept(() => {
            // Immediate update on accept
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }
            wasAccepted = true;
            quickPick.hide();
        });

        quickPick.onDidHide(() => {
            if (this.filterUpdateTimer) {
                clearTimeout(this.filterUpdateTimer);
            }

            // If user canceled (didn't accept), restore original filter values
            if (!wasAccepted) {
                this.categoryFilter = originalCategoryFilter;
                this.isCategoryFilterActive = originalIsCategoryFilterActive;
                this.refresh();
            }

            disposable.dispose();
            quickPick.dispose();
        });

        quickPick.show();
    }

    public clearCategoryFilter(): void {
        if (!this.workspace) {
            return;
        }

        // Reset to show all categories
        const allTasks = this.workspace.makeTasks;
        const allCategories = [...new Set(allTasks.map(task => task.category))];
        this.categoryFilter = new Set(allCategories);
        this.isCategoryFilterActive = false;
        this.refresh();
    }

    public get currentCategoryFilter(): Set<string> {
        return this.categoryFilter;
    }

    dispose(): void {
        // Clean up timer
        if (this.filterUpdateTimer) {
            clearTimeout(this.filterUpdateTimer);
        }

        this.subscriptions.forEach(sub => sub.dispose());
        this.subscriptions = [];
    }
}
