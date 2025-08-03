import * as vscode from 'vscode';
import * as path from 'path';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTaskProvider } from './cargoTaskProvider';
import { ProfilesTreeProvider } from './profilesTreeProvider';
import { TargetsTreeProvider } from './targetsTreeProvider';
import { WorkspaceTreeProvider } from './workspaceTreeProvider';
import { CargoProfile } from './cargoProfile';
import { CargoTarget, TargetActionType } from './cargoTarget';
import { CargoConfigurationReader } from './cargoConfigurationReader';

/**
 * Generates a unique correlation ID for tracking commands and operations
 */
function generateCorrelationId(): string {
    return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}

/**
 * The main extension manager that coordinates all cargo-tools functionality.
 * This is the singleton that manages the extension lifecycle and state.
 * Follows patterns from microsoft/vscode-cmake-tools for maintainability.
 */
export class CargoExtensionManager implements vscode.Disposable {
    private static instance?: CargoExtensionManager;
    private initializationPromise?: Promise<void>;

    // Core components
    private cargoWorkspace?: CargoWorkspace;
    private taskProvider?: CargoTaskProvider;
    private profilesTreeProvider?: ProfilesTreeProvider;
    private targetsTreeProvider?: TargetsTreeProvider;
    private workspaceTreeProvider?: WorkspaceTreeProvider;

    // Configuration management
    private readonly workspaceConfig: CargoConfigurationReader = CargoConfigurationReader.create();

    // Default target tracking for each action type
    private defaultTargets: Map<TargetActionType, CargoTarget | null> = new Map([
        [TargetActionType.Build, null],
        [TargetActionType.Run, null],
        [TargetActionType.Test, null],
        [TargetActionType.Bench, null]
    ]);

    // Subscriptions for cleanup
    private subscriptions: vscode.Disposable[] = [];
    private workspaceSubscriptions: vscode.Disposable[] = [];
    private commandsRegistered = false; // Guard flag to prevent double registration

    private constructor(private readonly extensionContext: vscode.ExtensionContext) { }

    /**
     * Create or get the singleton instance of the extension manager
     */
    static async create(context: vscode.ExtensionContext): Promise<CargoExtensionManager> {
        if (!CargoExtensionManager.instance) {
            CargoExtensionManager.instance = new CargoExtensionManager(context);
            CargoExtensionManager.instance.initializationPromise = CargoExtensionManager.instance.init();
            await CargoExtensionManager.instance.initializationPromise;
        }
        return CargoExtensionManager.instance;
    }

    /**
     * Get the current instance (should only be called after create)
     */
    static getInstance(): CargoExtensionManager {
        if (!CargoExtensionManager.instance) {
            throw new Error('Extension manager not initialized. Call create() first.');
        }
        return CargoExtensionManager.instance;
    }

    /**
     * Get the workspace configuration reader
     */
    public getWorkspaceConfig(): CargoConfigurationReader {
        return this.workspaceConfig;
    }

    /**
     * Initialize the extension manager and all components
     */
    public async init(): Promise<void> {
        // Set up configuration change listeners
        this.setupConfigurationSubscriptions();

        // Initialize core components
        await this.initializeComponents();

        // Register all commands
        this.registerCommands();

        // Set up workspace monitoring
        this.setupWorkspaceMonitoring();

        // Initialize workspace if available
        if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
            await this.initializeWorkspace();
        }
    }

    /**
     * Set up configuration change subscriptions following CMake Tools pattern
     */
    private setupConfigurationSubscriptions(): void {
        // Subscribe to configuration changes
        this.subscriptions.push(
            this.workspaceConfig.onChange('cargoPath', async (cargoPath: string) => {
                console.log(`Cargo path changed to: ${cargoPath}`);
                // Refresh workspace if needed
                if (this.cargoWorkspace) {
                    await this.cargoWorkspace.refreshTargets();
                }
            }),

            this.workspaceConfig.onChange('defaultProfile', async (profile: string) => {
                console.log(`Default profile changed to: ${profile}`);
                // Update active profile if needed
                if (this.cargoWorkspace) {
                    this.cargoWorkspace.setProfile(profile as CargoProfile);
                }
            }),

            this.workspaceConfig.onChange('excludeFolders', async (excludeFolders: string[]) => {
                console.log(`Exclude folders changed:`, excludeFolders);
                // Refresh workspace to respect new exclusions
                if (this.cargoWorkspace) {
                    await this.cargoWorkspace.refreshTargets();
                }
            }),

            // UI configuration monitoring
            this.workspaceConfig.onChange('treeView', async (treeViewConfig) => {
                console.log(`Tree view configuration changed:`, treeViewConfig);
                this.updateTreeViewVisibility(treeViewConfig);
            })
        );
    }

    /**
     * Initialize all extension components
     */
    private async initializeComponents(): Promise<void> {
        // Components will be initialized after workspace is available
        // since they depend on CargoWorkspace instance
    }

    /**
     * Initialize workspace-dependent components
     */
    private async initializeWorkspaceComponents(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Initialize tree providers with workspace
        this.profilesTreeProvider = new ProfilesTreeProvider(this.cargoWorkspace);
        this.targetsTreeProvider = new TargetsTreeProvider(this.cargoWorkspace);
        this.workspaceTreeProvider = new WorkspaceTreeProvider(this.cargoWorkspace);

        // Initialize task provider
        this.taskProvider = new CargoTaskProvider(this.cargoWorkspace, this.workspaceConfig);
        const taskProviderDisposable = vscode.tasks.registerTaskProvider('cargo', this.taskProvider);
        this.subscriptions.push(taskProviderDisposable);
    }

    /**
     * Register all extension commands with error handling wrapper
     */
    private registerCommands(): void {
        // Guard against multiple registrations
        if (this.commandsRegistered) {
            console.log('Commands already registered, skipping duplicate registration');
            return;
        }

        // Register command with improved CMake Tools-style wrapper
        const register = <K extends keyof CargoExtensionManager>(name: K) => {
            return vscode.commands.registerCommand(`cargo-tools.${name}`, async (...args: any[]) => {
                // Generate a unique ID that can be correlated in the log file
                const correlationId = generateCorrelationId();

                try {
                    console.log(`[${correlationId}] cargo-tools.${name} started`);

                    // Ensure we have a valid instance
                    if (!CargoExtensionManager.instance) {
                        throw new Error('Extension manager not initialized');
                    }

                    const command = (CargoExtensionManager.instance[name] as Function).bind(CargoExtensionManager.instance);
                    const result = await command(...args);

                    console.log(`[${correlationId}] cargo-tools.${name} completed`);
                    return result;
                } catch (error) {
                    console.error(`[${correlationId}] cargo-tools.${name} failed:`, error);

                    // Show user-friendly error message
                    const message = error instanceof Error ? error.message : String(error);
                    vscode.window.showErrorMessage(`Command failed: ${message}`);

                    throw error;
                }
            });
        };

        // List of commands to register - matches CMake Tools pattern
        const commands: (keyof CargoExtensionManager)[] = [
            'selectProfile',
            'refresh',
            'executeDefaultBuild',
            'executeDefaultRun',
            'executeDefaultTest',
            'executeDefaultBench',
            'setAsDefaultBuildTarget',
            'setAsDefaultRunTarget',
            'setAsDefaultTestTarget',
            'setAsDefaultBenchTarget',
            'executeBuildAction',
            'executeRunAction',
            'executeTestAction',
            'executeBenchAction'
        ];

        // Clear any existing command registrations to prevent duplicates
        console.log('Registering Cargo Tools commands...');

        // Register all commands with error handling
        for (const command of commands) {
            try {
                // Check if command already exists (safety check)
                const commandId = `cargo-tools.${command}`;

                const disposable = register(command);
                this.subscriptions.push(disposable);
                console.log(`Registered command: ${commandId}`);
            } catch (error) {
                console.error(`Failed to register command cargo-tools.${command}:`, error);

                // If it's a "command already exists" error, show a user-friendly message
                if (error instanceof Error && error.message.includes('already exists')) {
                    console.warn(`Command cargo-tools.${command} already exists - this may indicate an extension reload issue`);
                    vscode.window.showWarningMessage(
                        'Cargo Tools: Some commands may already be registered. Try reloading the window if you experience issues.',
                        'Reload Window'
                    ).then(selection => {
                        if (selection === 'Reload Window') {
                            vscode.commands.executeCommand('workbench.action.reloadWindow');
                        }
                    });
                } else {
                    // Re-throw other errors
                    throw error;
                }
            }
        }

        console.log(`Successfully registered ${commands.length} commands`);
        this.commandsRegistered = true; // Mark commands as registered
    }

    /**
     * Set up workspace folder monitoring
     */
    private setupWorkspaceMonitoring(): void {
        // Monitor workspace folder changes
        const workspaceFoldersChanged = vscode.workspace.onDidChangeWorkspaceFolders(async (event) => {
            console.log('Workspace folders changed');
            await this.handleWorkspaceFoldersChanged(event);
        });

        // Monitor file changes for Cargo.toml
        const fileWatcher = vscode.workspace.createFileSystemWatcher('**/Cargo.toml');
        const cargoTomlChanged = fileWatcher.onDidChange(async (uri) => {
            console.log('Cargo.toml changed:', uri.fsPath);
            await this.handleCargoTomlChanged(uri);
        });

        this.subscriptions.push(workspaceFoldersChanged, fileWatcher, cargoTomlChanged);
    }

    /**
     * Initialize the cargo workspace
     */
    private async initializeWorkspace(): Promise<void> {
        this.disposeWorkspaceSubscriptions();

        try {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                return;
            }

            this.cargoWorkspace = new CargoWorkspace(workspaceFolder.uri.fsPath);
            await this.cargoWorkspace.initialize();

            // Initialize workspace-dependent components
            await this.initializeWorkspaceComponents();

            // Set up workspace event subscriptions
            this.setupWorkspaceSubscriptions();

            console.log('Cargo workspace initialized successfully');
        } catch (error) {
            console.error('Failed to initialize cargo workspace:', error);
            vscode.window.showErrorMessage(`Failed to initialize cargo workspace: ${error}`);
        }
    }

    /**
     * Set up subscriptions to workspace events
     */
    private setupWorkspaceSubscriptions(): void {
        if (!this.cargoWorkspace) {
            return;
        }

        // Subscribe to workspace events - tree providers will auto-refresh via their own subscriptions
        const targetChangedSub = this.cargoWorkspace.onDidChangeTarget((target: CargoTarget | null) => {
            console.log('Target changed:', target?.name || 'none');
            // Status bar updates automatically via its own subscription
        });

        const profileChangedSub = this.cargoWorkspace.onDidChangeProfile((profile: CargoProfile) => {
            console.log('Profile changed:', CargoProfile.getDisplayName(profile));
            // Status bar updates automatically via its own subscription
        });

        const targetsChangedSub = this.cargoWorkspace.onDidChangeTargets((targets: CargoTarget[]) => {
            console.log('Targets changed, count:', targets.length);
            // Tree providers update automatically via their own subscriptions
        });

        this.workspaceSubscriptions.push(
            targetChangedSub,
            profileChangedSub,
            targetsChangedSub
        );
    }

    /**
     * Update all UI components with current workspace state
     */
    private async updateUIComponents(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Status bar is automatically updated through event subscriptions
        // Tree providers refresh themselves through event subscriptions
        console.log('UI components updated');
    }

    /**
     * Update status bar visibility based on configuration
     */
    /**
     * Update tree view visibility based on configuration
     */
    private updateTreeViewVisibility(config: { showProfiles: boolean; showTargets: boolean; showWorkspace: boolean; groupTargetsByKind: boolean }): void {
        // Note: Current tree view providers don't have visibility controls
        // This is a placeholder for future enhancement
        console.log('Tree view visibility updated:', config);
    }

    /**
     * Handle workspace folders changed event
     */
    private async handleWorkspaceFoldersChanged(event: vscode.WorkspaceFoldersChangeEvent): Promise<void> {
        // Re-initialize workspace if we have folders
        if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
            await this.initializeWorkspace();
        } else {
            // No workspace folders, clean up
            this.disposeWorkspaceSubscriptions();
            this.cargoWorkspace = undefined;
        }
    }

    /**
     * Handle Cargo.toml file changes
     */
    private async handleCargoTomlChanged(uri: vscode.Uri): Promise<void> {
        if (this.cargoWorkspace) {
            await this.cargoWorkspace.refresh();
            await this.updateUIComponents();
        }
    }

    /**
     * Generate a unique correlation ID for command tracking
     */
    private generateCorrelationId(): string {
        return Math.random().toString(36).substring(2, 8);
    }

    /**
     * Dispose workspace-specific subscriptions
     */
    private disposeWorkspaceSubscriptions(): void {
        this.workspaceSubscriptions.forEach(sub => sub.dispose());
        this.workspaceSubscriptions = [];
    }

    // Command implementations
    async selectProfile(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const profiles = [CargoProfile.dev, CargoProfile.release];
        const profileItems = profiles.map(profile => ({
            label: CargoProfile.getDisplayName(profile),
            profile: profile
        }));

        const selected = await vscode.window.showQuickPick(profileItems, {
            placeHolder: 'Select a build profile'
        });

        if (selected) {
            await this.cargoWorkspace.setProfile(selected.profile);
        }
    }

    async refresh(): Promise<void> {
        if (this.cargoWorkspace) {
            await this.cargoWorkspace.initialize();
            vscode.window.showInformationMessage('Cargo workspace refreshed');
        }
    }

    // Default target management

    /**
     * Set the default target for a specific action type
     */
    async setDefaultTarget(actionType: TargetActionType, target: CargoTarget | null): Promise<void> {
        if (target && !target.supportsActionType(actionType)) {
            throw new Error(`Target ${target.name} does not support action type ${actionType}`);
        }

        this.defaultTargets.set(actionType, target);

        // Store in workspace state for persistence
        const key = `cargo-tools.defaultTarget.${actionType}`;
        await this.extensionContext.workspaceState.update(key, target ? {
            name: target.name,
            packageName: target.packageName,
            kind: target.kind
        } : null);

        vscode.window.showInformationMessage(
            target
                ? `Set ${target.name} as default ${actionType} target`
                : `Cleared default ${actionType} target`
        );
    }

    /**
     * Get the default target for a specific action type
     */
    getDefaultTarget(actionType: TargetActionType): CargoTarget | null {
        return this.defaultTargets.get(actionType) || null;
    }

    /**
     * Execute an action using the default target for that action type
     */
    async executeDefaultAction(actionType: TargetActionType): Promise<void> {
        const defaultTarget = this.getDefaultTarget(actionType);

        if (!defaultTarget) {
            // No default target set, show selection dialog
            await this.selectAndExecuteAction(actionType);
            return;
        }

        await this.executeTargetAction(defaultTarget, actionType);
    }

    /**
     * Execute a specific action on a specific target
     */
    async executeTargetAction(target: CargoTarget, actionType: TargetActionType): Promise<void> {
        if (!target.supportsActionType(actionType)) {
            throw new Error(`Target ${target.name} does not support action type ${actionType}`);
        }

        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const command = target.getCargoCommand(actionType);
        const targetArgs = target.getTargetArgs(actionType);

        await this.executeCargoCommandForTarget(command, target);
    }

    /**
     * Show target selection dialog for a specific action type and execute
     */
    async selectAndExecuteAction(actionType: TargetActionType): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Filter targets that support the action type
        const supportedTargets = this.cargoWorkspace.targets.filter(target =>
            target.supportsActionType(actionType)
        );

        if (supportedTargets.length === 0) {
            vscode.window.showWarningMessage(`No targets support ${actionType} action`);
            return;
        }

        const items = supportedTargets.map(target => ({
            label: target.name,
            description: `${target.kind.join(', ')} in ${target.packageName || 'main'}`,
            detail: `Supports: ${target.supportedActionTypes.join(', ')}`,
            target: target
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: `Select a target to ${actionType}`
        });

        if (selected) {
            await this.executeTargetAction(selected.target, actionType);

            // Optionally ask to set as default
            const setAsDefault = await vscode.window.showInformationMessage(
                `Set ${selected.target.name} as default ${actionType} target?`,
                'Yes', 'No'
            );

            if (setAsDefault === 'Yes') {
                await this.setDefaultTarget(actionType, selected.target);
            }
        }
    }

    // Getters for components (for testing or advanced usage)
    getCargoWorkspace(): CargoWorkspace | undefined {
        return this.cargoWorkspace;
    }

    getTaskProvider(): CargoTaskProvider | undefined {
        return this.taskProvider;
    }

    /**
     * Check if we have a valid cargo project/workspace
     */
    hasCargoProject(): boolean {
        return this.cargoWorkspace !== undefined;
    }

    /**
     * Wait for the extension manager to be fully initialized
     */
    async waitForInitialization(): Promise<void> {
        if (this.initializationPromise) {
            await this.initializationPromise;
        }
    }

    /**
     * Execute a cargo command with proper error handling
     */
    private async executeCargoCommand(command: string): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const terminal = vscode.window.createTerminal({
            name: `Cargo ${command}`,
            cwd: this.cargoWorkspace.workspaceRoot
        });

        const cargoPath = this.workspaceConfig.cargoPath || 'cargo';
        const args = this.cargoWorkspace.getCargoArgs(command);
        const commandLine = `${cargoPath} ${args.join(' ')}`;

        terminal.sendText(commandLine);
        terminal.show();
    }

    // Command wrappers for new action-based commands

    async executeDefaultBuild(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Build);
    }

    async executeDefaultRun(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Run);
    }

    async executeDefaultTest(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Test);
    }

    async executeDefaultBench(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Bench);
    }

    // Context menu command wrappers for specific targets

    async setAsDefaultBuildTarget(target: CargoTarget): Promise<void> {
        await this.setDefaultTarget(TargetActionType.Build, target);
    }

    async setAsDefaultRunTarget(target: CargoTarget): Promise<void> {
        await this.setDefaultTarget(TargetActionType.Run, target);
    }

    async setAsDefaultTestTarget(target: CargoTarget): Promise<void> {
        await this.setDefaultTarget(TargetActionType.Test, target);
    }

    async setAsDefaultBenchTarget(target: CargoTarget): Promise<void> {
        await this.setDefaultTarget(TargetActionType.Bench, target);
    }

    async executeBuildAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Build);
    }

    async executeRunAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Run);
    }

    async executeTestAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Test);
    }

    async executeBenchAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Bench);
    }

    /**
     * Execute cargo command for a specific target with workspace awareness
     */
    private async executeCargoCommandForTarget(command: string, target: CargoTarget): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        // Get cargo args for the specific target
        const args = this.getCargoArgsForTarget(command, target);
        const cargoPath = this.workspaceConfig.cargoPath || 'cargo';

        // Get the correct working directory for the target
        const workingDirectory = this.getWorkingDirectoryForTarget(target);

        // Create terminal for command execution
        const terminal = vscode.window.createTerminal({
            name: `Cargo ${command} ${target.name}`,
            cwd: workingDirectory,
            env: { ...process.env, ...this.workspaceConfig.environment }
        });

        const commandLine = `${cargoPath} ${args.join(' ')}`;
        console.log(`Executing: ${commandLine} in ${workingDirectory}`);

        terminal.sendText(commandLine);
        terminal.show();

        // Show information message
        vscode.window.showInformationMessage(`Running ${command} for ${target.name}...`);
    }

    /**
     * Get cargo arguments for a specific target
     */
    private getCargoArgsForTarget(command: string, target: CargoTarget): string[] {
        const args = [command];

        // Add profile
        if (this.cargoWorkspace!.currentProfile === CargoProfile.release) {
            args.push('--release');
        }

        // For workspace projects, we need different logic depending on working directory
        // If we're executing in the package directory, we don't need -p flag
        // If we're executing from workspace root, we need -p flag
        const workingDirectory = this.getWorkingDirectoryForTarget(target);
        const isExecutingFromPackageDir = target.packagePath && workingDirectory === target.packagePath;

        if (target.packageName && this.cargoWorkspace!.isWorkspace && !isExecutingFromPackageDir) {
            args.push('-p', target.packageName);
        }

        // Add target-specific flags
        if (command !== 'clean' && target.kind && Array.isArray(target.kind)) {
            if (target.kind.includes('bin')) {
                args.push('--bin', target.name);
            } else if (target.kind.includes('lib')) {
                args.push('--lib');
            } else if (target.kind.includes('example')) {
                args.push('--example', target.name);
            } else if (target.kind.includes('test')) {
                args.push('--test', target.name);
            } else if (target.kind.includes('bench')) {
                args.push('--bench', target.name);
            }
        }

        // Add features and other configuration
        const features = this.workspaceConfig.features;
        if (features && Array.isArray(features) && features.length > 0) {
            args.push('--features', features.join(','));
        }

        if (this.workspaceConfig.allFeatures) {
            args.push('--all-features');
        }

        if (this.workspaceConfig.noDefaultFeatures) {
            args.push('--no-default-features');
        }

        // Add command-specific arguments from configuration
        const commandArgs = this.workspaceConfig[`${command}Args` as keyof typeof this.workspaceConfig] as string[] | undefined;
        if (commandArgs && Array.isArray(commandArgs)) {
            args.push(...commandArgs);
        }

        return args;
    }

    /**
     * Get the correct working directory for a target
     */
    private getWorkingDirectoryForTarget(target: CargoTarget): string {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        // For workspace members, use the package path if available
        if (target.packagePath && this.cargoWorkspace.isWorkspace) {
            return target.packagePath;
        }

        // For single-package projects or when package path is not available, use workspace root
        return this.cargoWorkspace.workspaceRoot;
    }

    /**
     * Get executable path for a specific target
     */
    private getTargetExecutablePathForTarget(target: CargoTarget): string {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const profile = this.cargoWorkspace.currentProfile === CargoProfile.release ? 'release' : 'debug';
        return path.join(this.cargoWorkspace.workspaceRoot, 'target', profile, target.name);
    }

    /**
     * Dispose of all resources - following CMake Tools disposal pattern
     */
    dispose(): void {
        console.log('Disposing Cargo Tools extension manager...');

        // Dispose workspace subscriptions first
        this.disposeWorkspaceSubscriptions();

        // Dispose all subscriptions (including commands)
        this.subscriptions.forEach(sub => {
            try {
                sub.dispose();
            } catch (error) {
                console.error('Error disposing subscription:', error);
            }
        });
        this.subscriptions.length = 0;

        // Dispose workspace configuration
        try {
            this.workspaceConfig.dispose();
        } catch (error) {
            console.error('Error disposing workspace config:', error);
        }

        // Clear workspace reference
        this.cargoWorkspace = undefined;

        // Reset command registration flag
        this.commandsRegistered = false;

        // Clear singleton instance
        CargoExtensionManager.instance = undefined;

        console.log('Cargo Tools extension manager disposed');
    }

    /**
     * Asynchronous disposal for long-running cleanup - following CMake Tools pattern
     */
    public async asyncDispose(): Promise<void> {
        console.log('Async disposing Cargo Tools extension manager...');

        // Perform any async cleanup if needed in the future
        // For now, just call synchronous dispose
        this.dispose();

        console.log('Cargo Tools extension manager async disposed');
    }
}
