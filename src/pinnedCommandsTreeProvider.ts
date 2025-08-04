import * as vscode from 'vscode';

export interface PinnedCommandsQuickPickItem extends vscode.QuickPickItem {
    command: string;
}

export class PinnedCommandNode extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly commandName: string,
        public readonly isVisible: boolean = true,
        public readonly tooltip?: string
    ) {
        super(label, vscode.TreeItemCollapsibleState.None);
        this.contextValue = 'pinnedCommand';
        this.tooltip = tooltip || `Execute ${this.label}`;

        // Set the command to execute when clicked
        this.command = {
            command: 'cargo-tools.executePinnedCommand',
            title: this.label,
            arguments: [this.commandName]
        };

        // Set icon based on command type
        this.iconPath = this.getIconForCommand(commandName);
    }

    async runThisCommand(): Promise<void> {
        await vscode.commands.executeCommand(this.commandName);
    }

    getTreeItem(): vscode.TreeItem {
        return this;
    }

    private getIconForCommand(commandName: string): vscode.ThemeIcon {
        if (commandName.includes('build')) {
            return new vscode.ThemeIcon('tools');
        } else if (commandName.includes('run')) {
            return new vscode.ThemeIcon('play');
        } else if (commandName.includes('test')) {
            return new vscode.ThemeIcon('beaker');
        } else if (commandName.includes('bench')) {
            return new vscode.ThemeIcon('dashboard');
        } else if (commandName.includes('clean')) {
            return new vscode.ThemeIcon('trash');
        } else if (commandName.includes('debug')) {
            return new vscode.ThemeIcon('debug-alt');
        } else if (commandName.includes('select') || commandName.includes('choose')) {
            return new vscode.ThemeIcon('settings-gear');
        } else if (commandName.includes('refresh')) {
            return new vscode.ThemeIcon('refresh');
        } else {
            return new vscode.ThemeIcon('terminal');
        }
    }
}

export class PinnedCommandsTreeProvider implements vscode.TreeDataProvider<PinnedCommandNode> {
    private _onDidChangeTreeData: vscode.EventEmitter<PinnedCommandNode | void> = new vscode.EventEmitter<PinnedCommandNode | void>();
    readonly onDidChangeTreeData: vscode.Event<PinnedCommandNode | void> = this._onDidChangeTreeData.event;

    private pinnedCommands: PinnedCommandNode[] = [];
    private pinnedCommandsKey: string = "cargoTools.pinnedCommands";
    private isInitialized = false;
    private context: vscode.ExtensionContext;

    constructor(context: vscode.ExtensionContext) {
        this.context = context;

        // Listen for configuration changes
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration(this.pinnedCommandsKey)) {
                this.doConfigureSettingsChange();
            }
        });
    }

    async initialize(): Promise<void> {
        this.pinnedCommands = []; // Reset to empty list

        const tryPushCommands = (commands: string[]) => {
            commands.forEach((commandName) => {
                const label = this.getCommandLabel(commandName);
                if (this.findNode(label) === -1) {
                    this.pinnedCommands.push(new PinnedCommandNode(label, commandName, true));
                }
            });
        };

        // Pin the commands that are requested from the user's settings
        const config = vscode.workspace.getConfiguration();
        if (config.has(this.pinnedCommandsKey)) {
            const settingsPinnedCommands = config.get(this.pinnedCommandsKey) as string[];
            if (settingsPinnedCommands && Array.isArray(settingsPinnedCommands)) {
                tryPushCommands(settingsPinnedCommands);
            }
        }

        // Pin commands that were pinned in the last session
        const lastSessionPinnedCommands = this.context.workspaceState.get(this.pinnedCommandsKey) as string[];
        if (lastSessionPinnedCommands && Array.isArray(lastSessionPinnedCommands)) {
            tryPushCommands(lastSessionPinnedCommands);
        }

        this.isInitialized = true;
    }

    async doConfigureSettingsChange(): Promise<void> {
        if (this.isInitialized) {
            await this.initialize();
            this.refresh();
        }
    }

    async addCommand(chosen: PinnedCommandsQuickPickItem): Promise<void> {
        const label = this.getCommandLabel(chosen.command);
        if (this.findNode(label) === -1) {
            this.pinnedCommands.push(new PinnedCommandNode(label, chosen.command, true));
            await this.refresh();
            await this.updateSettings();
        }
    }

    findNode(nodeLabel: string): number {
        return this.pinnedCommands.findIndex(node => node.label === nodeLabel);
    }

    async removeCommand(node: PinnedCommandNode): Promise<void> {
        const index = this.findNode(node.label as string);
        if (index !== -1) {
            this.pinnedCommands.splice(index, 1);
            await this.refresh();
            await this.updateSettings();
        }
    }

    async runCommand(node: PinnedCommandNode): Promise<void> {
        await node.runThisCommand();
    }

    getTreeItem(node: PinnedCommandNode): vscode.TreeItem {
        return node.getTreeItem();
    }

    async updateSettings(): Promise<void> {
        const pinnedCommands: string[] = this.pinnedCommands.map(x => x.commandName);
        await this.context.workspaceState.update(this.pinnedCommandsKey, pinnedCommands);
    }

    public async refresh(): Promise<void> {
        this._onDidChangeTreeData.fire();
    }

    async getChildren(): Promise<PinnedCommandNode[]> {
        if (!this.isInitialized) {
            await this.initialize();
        }
        return this.pinnedCommands.filter(x => x.isVisible);
    }

    private getCommandLabel(commandName: string): string {
        // Convert command names to readable labels
        const labels: { [key: string]: string } = {
            'cargo-tools.executeBuildAction': 'Build',
            'cargo-tools.executeRunAction': 'Run',
            'cargo-tools.executeTestAction': 'Test',
            'cargo-tools.executeBenchAction': 'Bench',
            'cargo-tools.buildDefaultTarget': 'Build Default Target',
            'cargo-tools.runDefaultTarget': 'Run Default Target',
            'cargo-tools.testDefaultTarget': 'Test Default Target',
            'cargo-tools.benchDefaultTarget': 'Bench Default Target',
            'cargo-tools.selectProfile': 'Select Build Profile',
            'cargo-tools.selectTarget': 'Select Build Target',
            'cargo-tools.refresh': 'Refresh',
            'cargo-tools.editConfiguration': 'Edit Configuration',
            'cargo-tools.runExample': 'Run Example',
            'cargo-tools.runTest': 'Run Test',
            'cargo-tools.runBench': 'Run Benchmark'
        };

        return labels[commandName] || commandName.replace('cargo-tools.', '').replace(/([A-Z])/g, ' $1').trim();
    }

    static getPinnableCommands(): string[] {
        return [
            'cargo-tools.executeBuildAction',
            'cargo-tools.executeRunAction',
            'cargo-tools.executeTestAction',
            'cargo-tools.executeBenchAction',
            'cargo-tools.buildDefaultTarget',
            'cargo-tools.runDefaultTarget',
            'cargo-tools.testDefaultTarget',
            'cargo-tools.benchDefaultTarget',
            'cargo-tools.selectProfile',
            'cargo-tools.selectTarget',
            'cargo-tools.refresh',
            'cargo-tools.editConfiguration',
            'cargo-tools.runExample',
            'cargo-tools.runTest',
            'cargo-tools.runBench'
        ];
    }
}

export class PinnedCommands {
    private treeDataProvider: PinnedCommandsTreeProvider;
    private disposables: vscode.Disposable[] = [];

    constructor(context: vscode.ExtensionContext) {
        this.treeDataProvider = new PinnedCommandsTreeProvider(context);
        this.disposables.push(...[
            // Commands for pinned commands items
            vscode.commands.registerCommand('cargo-tools.pinnedCommands.add', async () => {
                const chosen = await this.showPinnableCommands();
                if (chosen !== null) {
                    await this.treeDataProvider.addCommand(chosen);
                }
            }),
            vscode.commands.registerCommand('cargo-tools.pinnedCommands.remove', async (what: PinnedCommandNode) => {
                await this.treeDataProvider.removeCommand(what);
            }),
            vscode.commands.registerCommand('cargo-tools.pinnedCommands.run', async (what: PinnedCommandNode) => {
                await this.treeDataProvider.runCommand(what);
            }),
            vscode.commands.registerCommand('cargo-tools.executePinnedCommand', async (commandName: string) => {
                await vscode.commands.executeCommand(commandName);
            })
        ]);
    }

    /**
     * Show List of All Commands that can be pinned
     */
    async showPinnableCommands(): Promise<PinnedCommandsQuickPickItem | null> {
        const items = PinnedCommandsTreeProvider.getPinnableCommands().map((commandName) => ({
            command: commandName,
            label: this.treeDataProvider['getCommandLabel'](commandName),
            description: commandName
        } as PinnedCommandsQuickPickItem));

        const chosenItem = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a Cargo Tools command to pin'
        });

        if (!chosenItem) {
            return null;
        }

        return chosenItem;
    }

    refresh(): Promise<any> {
        return this.treeDataProvider.refresh();
    }

    dispose(): void {
        vscode.Disposable.from(...this.disposables).dispose();
    }

    getTreeDataProvider(): PinnedCommandsTreeProvider {
        return this.treeDataProvider;
    }
}
