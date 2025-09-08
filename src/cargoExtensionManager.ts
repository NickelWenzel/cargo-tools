import * as vscode from 'vscode';
import * as path from 'path';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTaskProvider } from './cargoTaskProvider';
import { CargoProfile } from './cargoProfile';
import { CargoTarget, TargetActionType } from './cargoTarget';
import { CargoConfigurationReader } from './cargoConfigurationReader';
import { StatusBarProvider } from './statusBarProvider';
import { ProjectOutlineTreeProvider } from './projectOutlineTreeProvider';
import { ProjectStatusTreeProvider } from './projectStatusTreeProvider';
import { StateManager } from './stateManager';

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
    private statusBarProvider?: StatusBarProvider;
    private projectOutlineTreeProvider?: ProjectOutlineTreeProvider;
    private projectStatusTreeProvider?: ProjectStatusTreeProvider;
    private stateManager?: StateManager;

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
    private docsTerminal?: vscode.Terminal; // Reusable terminal for cargo doc commands

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
            this.workspaceConfig.onChange('cargoCommand', async (cargoCommand: string) => {
                console.log(`Cargo command changed to: ${cargoCommand}`);
                // Refresh workspace if needed
                if (this.cargoWorkspace) {
                    await this.cargoWorkspace.refreshTargets();
                }
            }),

            this.workspaceConfig.onChange('cargoPath', async (cargoPath: string) => {
                console.log(`Cargo path changed to: ${cargoPath}`);
                // Refresh workspace if needed
                if (this.cargoWorkspace) {
                    await this.cargoWorkspace.refreshTargets();
                }
            }),

            this.workspaceConfig.onChange('useRustAnalyzerEnvAndArgs', async (enabled: boolean) => {
                console.log(`Rust-analyzer integration changed to: ${enabled}`);
                // Refresh workspace when rust-analyzer integration is toggled
                if (this.cargoWorkspace) {
                    await this.cargoWorkspace.refreshTargets();
                }
            }),

            // Listen for rust-analyzer configuration changes when integration is enabled
            vscode.workspace.onDidChangeConfiguration(e => {
                if (this.workspaceConfig.useRustAnalyzerEnvAndArgs &&
                    (e.affectsConfiguration('rust-analyzer.cargo.extraArgs') ||
                        e.affectsConfiguration('rust-analyzer.cargo.extraEnv') ||
                        e.affectsConfiguration('rust-analyzer.runnables.extraArgs') ||
                        e.affectsConfiguration('rust-analyzer.runnables.extraTestBinaryArgs'))) {
                    console.log('Rust-analyzer settings changed, refreshing workspace...');
                    if (this.cargoWorkspace) {
                        this.cargoWorkspace.refreshTargets().catch(err => {
                            console.error('Failed to refresh workspace after rust-analyzer config change:', err);
                        });
                    }
                }
            }),

            this.workspaceConfig.onChange('defaultProfile', async (profile: string) => {
                console.log(`Default profile changed to: ${profile}`);
                // Update active profile if needed
                if (this.cargoWorkspace) {
                    this.cargoWorkspace.setProfile(CargoProfile.fromString(profile));
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
        // Initialize status bar provider (doesn't depend on workspace)
        this.statusBarProvider = new StatusBarProvider(this.workspaceConfig);
        this.subscriptions.push(this.statusBarProvider);

        // Components that depend on workspace will be initialized after workspace is available
    }

    /**
     * Initialize workspace-dependent components
     */
    private async initializeWorkspaceComponents(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Initialize task provider
        this.taskProvider = new CargoTaskProvider(this.cargoWorkspace, this.workspaceConfig);
        const taskProviderDisposable = vscode.tasks.registerTaskProvider('cargo', this.taskProvider);
        this.subscriptions.push(taskProviderDisposable);
    }

    /**
     * Register tree providers with the extension manager for command access
     */
    public registerTreeProviders(projectOutlineProvider: ProjectOutlineTreeProvider, projectStatusProvider: ProjectStatusTreeProvider): void {
        this.projectOutlineTreeProvider = projectOutlineProvider;
        this.projectStatusTreeProvider = projectStatusProvider;

        // Set up state management for tree providers if we have a state manager
        if (this.stateManager) {
            projectOutlineProvider.setStateManager(this.stateManager);
            // Load persisted state for the project outline provider
            projectOutlineProvider.loadPersistedState();
        }
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
            'selectPackage',
            'selectBuildTarget',
            'selectRunTarget',
            'selectBenchmarkTarget',
            'selectPlatformTarget',
            'installPlatformTarget',
            'setRustAnalyzerCheckTargets',
            'buildDocs',
            'setBuildTarget',
            'setRunTarget',
            'setTestTarget',
            'setBenchTarget',
            'selectFeatures',
            'toggleFeature',
            'refresh',
            'executeDefaultBuild',
            'executeDefaultRun',
            'executeDefaultDebug',
            'executeDefaultTest',
            'executeDefaultBench',
            'executeBuildAction',
            'executeRunAction',
            'executeDebugAction',
            'executeTestAction',
            'executeBenchAction'
        ];

        // Project Outline specific commands
        const projectOutlineCommands = [
            'projectOutline.selectPackage',
            'projectOutline.unselectPackage',
            'projectOutline.setBuildTarget',
            'projectOutline.unsetBuildTarget',
            'projectOutline.setRunTarget',
            'projectOutline.unsetRunTarget',
            'projectOutline.setBenchmarkTarget',
            'projectOutline.unsetBenchmarkTarget',
            'projectOutline.toggleFeature',
            'projectOutline.buildPackage',
            'projectOutline.testPackage',
            'projectOutline.buildTarget',
            'projectOutline.runTarget',
            'projectOutline.debugTarget',
            'projectOutline.benchTarget',
            'projectOutline.setWorkspaceMemberFilter',
            'projectOutline.clearWorkspaceMemberFilter',
            'projectOutline.showTargetTypeFilter',
            'projectOutline.clearTargetTypeFilter',
            'projectOutline.clearAllFilters',
            'projectOutline.toggleWorkspaceMemberGrouping'
        ];

        // Project Status execution commands
        const projectStatusCommands = [
            'projectStatus.build',
            'projectStatus.run',
            'projectStatus.debug',
            'projectStatus.test',
            'projectStatus.bench'
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

        // Register project outline specific commands manually
        for (const commandName of projectOutlineCommands) {
            try {
                const commandId = `cargo-tools.${commandName}`;
                const method = commandName.replace(/\./g, '_'); // Convert dots to underscores for method names

                const disposable = vscode.commands.registerCommand(commandId, async (...args: any[]) => {
                    const correlationId = generateCorrelationId();
                    try {
                        console.log(`[${correlationId}] ${commandId} started`);
                        if (!CargoExtensionManager.instance) {
                            throw new Error('Extension manager not initialized');
                        }

                        const command = (CargoExtensionManager.instance as any)[method];
                        if (typeof command === 'function') {
                            const result = await command.call(CargoExtensionManager.instance, ...args);
                            console.log(`[${correlationId}] ${commandId} completed`);
                            return result;
                        } else {
                            throw new Error(`Command method ${method} not found`);
                        }
                    } catch (error) {
                        console.error(`[${correlationId}] ${commandId} failed:`, error);
                        const message = error instanceof Error ? error.message : String(error);
                        vscode.window.showErrorMessage(`Command failed: ${message}`);
                        throw error;
                    }
                });

                this.subscriptions.push(disposable);
                console.log(`Registered project outline command: ${commandId}`);
            } catch (error) {
                console.error(`Failed to register project outline command ${commandName}:`, error);
            }
        }

        // Register project status execution commands manually
        for (const commandName of projectStatusCommands) {
            try {
                const commandId = `cargo-tools.${commandName}`;
                const method = commandName.replace(/\./g, '_'); // Convert dots to underscores for method names

                const disposable = vscode.commands.registerCommand(commandId, async (...args: any[]) => {
                    const correlationId = generateCorrelationId();
                    try {
                        console.log(`[${correlationId}] ${commandId} started`);
                        if (!CargoExtensionManager.instance) {
                            throw new Error('Extension manager not initialized');
                        }

                        const command = (CargoExtensionManager.instance as any)[method];
                        if (typeof command === 'function') {
                            const result = await command.call(CargoExtensionManager.instance, ...args);
                            console.log(`[${correlationId}] ${commandId} completed`);
                            return result;
                        } else {
                            throw new Error(`Command method ${method} not found`);
                        }
                    } catch (error) {
                        console.error(`[${correlationId}] ${commandId} failed:`, error);
                        const message = error instanceof Error ? error.message : String(error);
                        vscode.window.showErrorMessage(`Command failed: ${message}`);
                        throw error;
                    }
                });

                this.subscriptions.push(disposable);
                console.log(`Registered project status command: ${commandId}`);
            } catch (error) {
                console.error(`Failed to register project status command ${commandName}:`, error);
            }
        }

        console.log(`Successfully registered ${commands.length} main commands + ${projectOutlineCommands.length} outline commands + ${projectStatusCommands.length} status commands`);
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

            // Create state manager for this workspace
            this.stateManager = new StateManager(this.extensionContext, workspaceFolder);

            this.cargoWorkspace = new CargoWorkspace(workspaceFolder.uri.fsPath);
            await this.cargoWorkspace.initialize();

            // Load persisted state
            await this.loadPersistedState(workspaceFolder);

            // Initialize workspace-dependent components
            await this.initializeWorkspaceComponents();

            // Set up state management for tree providers if they're already registered
            if (this.projectOutlineTreeProvider) {
                this.projectOutlineTreeProvider.setStateManager(this.stateManager);
                await this.projectOutlineTreeProvider.loadPersistedState();
            }

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
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const profileChangedSub = this.cargoWorkspace.onDidChangeProfile((profile: CargoProfile) => {
            console.log('Profile changed:', CargoProfile.getDisplayName(profile));
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const packageChangedSub = this.cargoWorkspace.onDidChangeSelectedPackage((packageName: string | undefined) => {
            console.log('Package changed:', packageName || 'No selection');
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const buildTargetChangedSub = this.cargoWorkspace.onDidChangeSelectedBuildTarget((targetName: string | null) => {
            console.log('Build target changed:', targetName || 'none');
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const runTargetChangedSub = this.cargoWorkspace.onDidChangeSelectedRunTarget((targetName: string | null) => {
            console.log('Run target changed:', targetName || 'none');
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const benchmarkTargetChangedSub = this.cargoWorkspace.onDidChangeSelectedBenchmarkTarget((targetName: string | null) => {
            console.log('Benchmark target changed:', targetName || 'none');
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const platformTargetChangedSub = this.cargoWorkspace.onDidChangeSelectedPlatformTarget((targetTriple: string | null) => {
            console.log('Platform target changed:', targetTriple || 'none');
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const featuresChangedSub = this.cargoWorkspace.onDidChangeSelectedFeatures((features: Set<string>) => {
            console.log('Features changed:', Array.from(features));
            this.updateStatusBar();
            this.saveCurrentState(); // Persist state changes
        });

        const targetsChangedSub = this.cargoWorkspace.onDidChangeTargets((targets: CargoTarget[]) => {
            console.log('Targets changed, count:', targets.length);
            this.updateStatusBar();
        });

        this.workspaceSubscriptions.push(
            targetChangedSub,
            profileChangedSub,
            packageChangedSub,
            buildTargetChangedSub,
            runTargetChangedSub,
            benchmarkTargetChangedSub,
            platformTargetChangedSub,
            featuresChangedSub,
            targetsChangedSub
        );

        // Initial status bar sync with current workspace state
        this.updateStatusBar();
    }

    /**
     * Update all UI components with current workspace state
     */
    private async updateUIComponents(): Promise<void> {
        if (!this.cargoWorkspace || !this.statusBarProvider) {
            return;
        }

        // Update status bar with current selections
        this.updateStatusBar();

        // Tree providers refresh themselves through event subscriptions
        console.log('UI components updated');
    }

    /**
     * Update status bar with current workspace state
     */
    private updateStatusBar(): void {
        if (!this.cargoWorkspace || !this.statusBarProvider) {
            return;
        }

        // Update profile
        this.statusBarProvider.setProfileName(this.cargoWorkspace.currentProfile);

        // Update package
        this.statusBarProvider.setPackageName(this.cargoWorkspace.selectedPackage);

        // Update targets
        const buildTargetName = this.cargoWorkspace.selectedBuildTarget;
        const displayBuildTarget = buildTargetName === 'lib' ? 'lib' : buildTargetName;
        this.statusBarProvider.setBuildTargetName(displayBuildTarget);
        this.statusBarProvider.setRunTargetName(this.cargoWorkspace.selectedRunTarget);
        this.statusBarProvider.setBenchmarkTargetName(this.cargoWorkspace.selectedBenchmarkTarget);
        this.statusBarProvider.setPlatformTargetName(this.cargoWorkspace.selectedPlatformTarget);

        // Update features
        const selectedFeatures = this.cargoWorkspace.selectedFeatures;
        let featuresText = '';
        if (selectedFeatures.has('all-features')) {
            featuresText = 'all-features';
        } else if (selectedFeatures.size === 0) {
            featuresText = 'no features';
        } else {
            featuresText = Array.from(selectedFeatures).join(', ');
        }
        this.statusBarProvider.setFeaturesText(featuresText);

        // Update button visibility based on package selection
        const packageSelected = this.cargoWorkspace.selectedPackage !== undefined;
        this.statusBarProvider.updateTargetButtonsVisibility(packageSelected);
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

    /**
     * Load persisted state from previous sessions and apply it to the workspace
     */
    private async loadPersistedState(workspaceFolder: vscode.WorkspaceFolder): Promise<void> {
        if (!this.stateManager || !this.cargoWorkspace) {
            return;
        }

        try {
            const folderName = workspaceFolder.name;
            const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

            // Load Project Status View state
            const selectedPackage = this.stateManager.getSelectedPackage(folderName, isMultiProject);
            const selectedBuildTarget = this.stateManager.getSelectedBuildTarget(folderName, isMultiProject);
            const selectedRunTarget = this.stateManager.getSelectedRunTarget(folderName, isMultiProject);
            const selectedBenchmarkTarget = this.stateManager.getSelectedBenchmarkTarget(folderName, isMultiProject);
            const selectedPlatformTarget = this.stateManager.getSelectedPlatformTarget(folderName, isMultiProject);
            const selectedFeatures = this.stateManager.getSelectedFeatures(folderName, isMultiProject);
            const selectedProfile = this.stateManager.getSelectedProfile(folderName, isMultiProject);

            // Apply persisted state to workspace
            if (selectedPackage !== undefined) {
                this.cargoWorkspace.setSelectedPackage(selectedPackage);
            }
            if (selectedBuildTarget !== null) {
                this.cargoWorkspace.setSelectedBuildTarget(selectedBuildTarget);
            }
            if (selectedRunTarget !== null) {
                this.cargoWorkspace.setSelectedRunTarget(selectedRunTarget);
            }
            if (selectedBenchmarkTarget !== null) {
                this.cargoWorkspace.setSelectedBenchmarkTarget(selectedBenchmarkTarget);
            }
            if (selectedPlatformTarget !== null) {
                this.cargoWorkspace.setSelectedPlatformTarget(selectedPlatformTarget);
            }
            if (selectedFeatures.length > 0) {
                this.cargoWorkspace.setSelectedFeatures(new Set(selectedFeatures));
            }
            if (selectedProfile !== null) {
                const profile = CargoProfile.fromString(selectedProfile);
                if (profile) {
                    this.cargoWorkspace.setProfile(profile);
                }
            }

            // Load Project Outline View state
            const groupByWorkspaceMember = this.stateManager.getGroupByWorkspaceMember(folderName, isMultiProject);
            const workspaceMemberFilter = this.stateManager.getWorkspaceMemberFilter(folderName, isMultiProject);
            const targetTypeFilter = this.stateManager.getTargetTypeFilter(folderName, isMultiProject);
            const isTargetTypeFilterActive = this.stateManager.getIsTargetTypeFilterActive(folderName, isMultiProject);
            const showFeatures = this.stateManager.getShowFeatures(folderName, isMultiProject);

            // Apply filter state to project outline tree provider (will be done when initializing components)
            console.log('Loaded persisted state for workspace:', folderName);
        } catch (error) {
            console.error('Failed to load persisted state:', error);
        }
    }

    /**
     * Save current state when it changes
     */
    private async saveCurrentState(): Promise<void> {
        if (!this.stateManager || !this.cargoWorkspace) {
            return;
        }

        try {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                return;
            }

            const folderName = workspaceFolder.name;
            const isMultiProject = (vscode.workspace.workspaceFolders?.length || 0) > 1;

            // Save Project Status View state
            await this.stateManager.setSelectedPackage(folderName, this.cargoWorkspace.selectedPackage, isMultiProject);
            await this.stateManager.setSelectedBuildTarget(folderName, this.cargoWorkspace.selectedBuildTarget, isMultiProject);
            await this.stateManager.setSelectedRunTarget(folderName, this.cargoWorkspace.selectedRunTarget, isMultiProject);
            await this.stateManager.setSelectedBenchmarkTarget(folderName, this.cargoWorkspace.selectedBenchmarkTarget, isMultiProject);
            await this.stateManager.setSelectedPlatformTarget(folderName, this.cargoWorkspace.selectedPlatformTarget, isMultiProject);
            await this.stateManager.setSelectedFeatures(folderName, Array.from(this.cargoWorkspace.selectedFeatures), isMultiProject);
            await this.stateManager.setSelectedProfile(folderName, CargoProfile.toString(this.cargoWorkspace.currentProfile), isMultiProject);

            // Save Project Outline View state (will be saved when tree provider state changes)
        } catch (error) {
            console.error('Failed to save current state:', error);
        }
    }

    // Command implementations
    async selectProfile(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const profiles = CargoProfile.getAllProfiles();
        const profileItems = profiles.map(profile => ({
            label: CargoProfile.getDisplayName(profile),
            description: CargoProfile.getDescription(profile),
            profile: profile
        }));

        const selected = await vscode.window.showQuickPick(profileItems, {
            placeHolder: 'Select a build profile'
        });

        if (selected) {
            await this.cargoWorkspace.setProfile(selected.profile);
        }
    }

    async selectPackage(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const packageItems: { label: string; package?: string }[] = [];

        // Always add "No selection" option first
        packageItems.push({
            label: 'No selection',
            package: undefined // No -p flag
        });

        if (this.cargoWorkspace.isWorkspace) {
            // Multi-package workspace
            // Add individual packages
            for (const member of this.cargoWorkspace.workspaceMembers) {
                packageItems.push({
                    label: member,
                    package: member
                });
            }
        } else {
            // Single package - still allow "No selection" vs selecting the package
            const packageName = this.cargoWorkspace.projectName;
            packageItems.push({
                label: packageName,
                package: packageName
            });
        }

        const selected = await vscode.window.showQuickPick(packageItems, {
            placeHolder: 'Select a package to build'
        });

        if (selected) {
            // Set the selected package in the workspace
            await this.cargoWorkspace.setSelectedPackage(selected.package);
        }
    }

    async selectBuildTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const items: vscode.QuickPickItem[] = [];
        const selectedPackage = this.cargoWorkspace.selectedPackage;

        // Always add "No selection" option first
        items.push({
            label: 'No selection',
            description: 'Build default targets (no target specification)',
            detail: 'No target selection'
        });

        if (!selectedPackage) {
            // No Package Selected - no additional options beyond "No selection"
        } else {
            // Specific Package Selected - show categorized targets
            const packageTargets = this.getTargetsForPackage(selectedPackage);
            const targetsByType = this.groupTargetsByType(packageTargets);

            // Add library if exists
            if (targetsByType.has('lib')) {
                items.push({
                    label: 'lib',
                    description: 'Build library (--lib)',
                    detail: 'Library target'
                });
            }

            // Add binaries
            if (targetsByType.has('bin')) {
                const binTargets = targetsByType.get('bin')!;
                for (const target of binTargets) {
                    items.push({
                        label: target.name,
                        description: `Build binary: ${target.name} (--bin ${target.name})`,
                        detail: 'Binary target'
                    });
                }
            }

            // Add examples
            if (targetsByType.has('example')) {
                const exampleTargets = targetsByType.get('example')!;
                for (const target of exampleTargets) {
                    items.push({
                        label: target.name,
                        description: `Build example: ${target.name} (--example ${target.name})`,
                        detail: 'Example target'
                    });
                }
            }

            // Add benchmarks
            if (targetsByType.has('bench')) {
                const benchTargets = targetsByType.get('bench')!;
                for (const target of benchTargets) {
                    items.push({
                        label: target.name,
                        description: `Build benchmark: ${target.name} (--bench ${target.name})`,
                        detail: 'Benchmark target'
                    });
                }
            }
        }

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a build target'
        });

        if (selected) {
            // Store build target selection - handle "No selection" case
            const targetToSet = selected.label === 'No selection' ? null : selected.label;
            this.cargoWorkspace.setSelectedBuildTarget(targetToSet);
        }
    }

    async selectRunTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const items: vscode.QuickPickItem[] = [];
        const selectedPackage = this.cargoWorkspace.selectedPackage;

        if (!selectedPackage) {
            // No Package Selected - disabled
            vscode.window.showWarningMessage('Select a specific package to run targets');
            return;
        } else {
            // Specific Package Selected - show bins and examples
            const packageTargets = this.getTargetsForPackage(selectedPackage);
            const targetsByType = this.groupTargetsByType(packageTargets);

            // Add binaries
            if (targetsByType.has('bin')) {
                const binTargets = targetsByType.get('bin')!;
                for (const target of binTargets) {
                    items.push({
                        label: target.name,
                        description: `Run binary: ${target.name} (--bin ${target.name})`,
                        detail: 'Binary target'
                    });
                }
            }

            // Add examples
            if (targetsByType.has('example')) {
                const exampleTargets = targetsByType.get('example')!;
                for (const target of exampleTargets) {
                    items.push({
                        label: target.name,
                        description: `Run example: ${target.name} (--example ${target.name})`,
                        detail: 'Example target'
                    });
                }
            }

            if (items.length === 0) {
                vscode.window.showInformationMessage('No runnable targets in selected package');
                return;
            }
        }

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a target to run'
        });

        if (selected) {
            // Store run target selection
            this.cargoWorkspace.setSelectedRunTarget(selected.label);
        }
    }

    async selectBenchmarkTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const items: vscode.QuickPickItem[] = [];
        const selectedPackage = this.cargoWorkspace.selectedPackage;

        // Always add "No selection" option first
        items.push({
            label: 'No selection',
            description: 'Run all benchmarks (no target specification)',
            detail: 'No benchmark target selection'
        });

        if (!selectedPackage) {
            vscode.window.showWarningMessage('Select a specific package to run benchmark targets');
            return;
        } else {
            // Specific Package Selected - show benchmarks from selected package
            const packageTargets = this.getTargetsForPackage(selectedPackage);
            const targetsByType = this.groupTargetsByType(packageTargets);

            // Add benchmarks
            if (targetsByType.has('bench')) {
                const benchTargets = targetsByType.get('bench')!;
                for (const target of benchTargets) {
                    items.push({
                        label: target.name,
                        description: `Run benchmark: ${target.name} (--bench ${target.name})`,
                        detail: 'Benchmark target'
                    });
                }
            } else {
                vscode.window.showInformationMessage('No benchmark targets in selected package');
                return;
            }
        }

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a benchmark target'
        });

        if (selected) {
            // Store benchmark target selection - handle "No selection" case
            const targetToSet = selected.label === 'No selection' ? null : selected.label;
            this.cargoWorkspace.setSelectedBenchmarkTarget(targetToSet);
        }
    }

    async selectPlatformTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const items: vscode.QuickPickItem[] = [];

        // Always add "No selection" option first
        items.push({
            label: 'No selection',
            description: 'Use default host target',
            detail: 'No platform target selection'
        });

        try {
            // Get installed platform targets
            const installedTargets = await this.cargoWorkspace.getInstalledPlatformTargets();

            // Add installed targets to selection
            for (const target of installedTargets) {
                items.push({
                    label: target,
                    description: `Cross-compile to ${target}`,
                    detail: 'Installed platform target'
                });
            }

            if (installedTargets.length === 0) {
                items.push({
                    label: '(No installed targets found)',
                    description: 'Use "Install Platform Target" to add targets',
                    detail: 'Run rustup target list --installed'
                });
            }

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to get platform targets: ${error}`);
            return;
        }

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a platform target'
        });

        if (selected && selected.label !== '(No installed targets found)') {
            // Store platform target selection - handle "No selection" case
            const targetToSet = selected.label === 'No selection' ? null : selected.label;
            this.cargoWorkspace.setSelectedPlatformTarget(targetToSet);
        }
    }

    async installPlatformTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        try {
            // Get available uninstalled targets
            const uninstalledTargets = await this.cargoWorkspace.getAvailableUninstalledPlatformTargets();

            if (uninstalledTargets.length === 0) {
                vscode.window.showInformationMessage('All available platform targets are already installed');
                return;
            }

            // Create QuickPickItems for uninstalled targets
            const items: vscode.QuickPickItem[] = uninstalledTargets.map(target => ({
                label: target,
                description: `Install target for ${target}`,
                detail: 'Available platform target'
            }));

            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: 'Select a platform target to install'
            });

            if (selected) {
                // Show progress while installing
                await vscode.window.withProgress({
                    location: vscode.ProgressLocation.Notification,
                    title: `Installing platform target ${selected.label}...`,
                    cancellable: false
                }, async (progress) => {
                    const success = await this.cargoWorkspace!.installPlatformTarget(selected.label);

                    if (success) {
                        vscode.window.showInformationMessage(`Successfully installed platform target: ${selected.label}`);
                    } else {
                        vscode.window.showErrorMessage(`Failed to install platform target: ${selected.label}`);
                    }
                });
            }

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to get available platform targets: ${error}`);
        }
    }

    async setRustAnalyzerCheckTargets(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        try {
            // Get installed platform targets
            const installedTargets = await this.cargoWorkspace.getInstalledPlatformTargets();

            if (installedTargets.length === 0) {
                vscode.window.showInformationMessage('No installed platform targets found. Install targets first using "Install Platform Target" command.');
                return;
            }

            // Get current rust-analyzer check targets setting
            const rustAnalyzerConfig = vscode.workspace.getConfiguration('rust-analyzer');
            const currentTargets: string[] = rustAnalyzerConfig.get('check.targets', []) || [];

            // Create QuickPickItems with checkboxes for installed targets
            const items: vscode.QuickPickItem[] = installedTargets.map(target => ({
                label: target,
                description: currentTargets.includes(target) ? '✓ Selected' : '',
                detail: `Platform target for rust-analyzer checks`,
                picked: currentTargets.includes(target)
            }));

            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: 'Select platform targets for rust-analyzer checks (use Space to toggle)',
                canPickMany: true,
                ignoreFocusOut: true
            });

            if (selected !== undefined) {
                // Update rust-analyzer check targets setting
                const newTargets = selected.map(item => item.label);

                if (newTargets.length === 0) {
                    // Remove setting if no targets selected
                    await rustAnalyzerConfig.update('check.targets', undefined, vscode.ConfigurationTarget.Workspace);
                } else {
                    // Set the new targets
                    await rustAnalyzerConfig.update('check.targets', newTargets, vscode.ConfigurationTarget.Workspace);
                }

                const message = newTargets.length === 0
                    ? 'Cleared rust-analyzer check targets'
                    : `Set rust-analyzer check targets: ${newTargets.join(', ')}`;
                vscode.window.showInformationMessage(message);
            }

        } catch (error) {
            vscode.window.showErrorMessage(`Failed to set rust-analyzer check targets: ${error}`);
        }
    }

    async buildDocs(): Promise<void> {
        if (!this.cargoWorkspace) {
            vscode.window.showErrorMessage('No Cargo workspace found');
            return;
        }

        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Building documentation...',
                cancellable: false
            }, async (progress) => {
                // Execute cargo doc command using a reusable terminal
                const terminal = this.getOrCreateDocsTerminal();

                const cargoCommand = this.workspaceConfig.cargoCommand || 'cargo';

                // Split cargoCommand at whitespaces - first part is command, rest are additional args
                const commandParts = cargoCommand.trim().split(/\s+/);
                const actualCommand = commandParts[0];
                const cargoCommandArgs = commandParts.slice(1);

                // Combine command args with the doc command
                const fullArgs = [...cargoCommandArgs, 'doc', '--no-deps', '--release'];
                const command = `${actualCommand} ${fullArgs.join(' ')}`;
                terminal.sendText(command);
                terminal.show();

                vscode.window.showInformationMessage('Building documentation with cargo doc --no-deps --release');
            });
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to build documentation: ${error}`);
        }
    }

    /**
     * Get or create a reusable terminal for cargo doc commands
     */
    private getOrCreateDocsTerminal(): vscode.Terminal {
        // Check if existing terminal is still valid (not closed)
        if (this.docsTerminal && this.docsTerminal.exitStatus === undefined) {
            return this.docsTerminal;
        }

        // Build environment variables for doc commands
        const env = this.buildCargoEnvironment('doc');

        // Create new terminal if none exists or previous one was closed
        this.docsTerminal = vscode.window.createTerminal({
            name: 'Cargo doc',
            cwd: this.cargoWorkspace?.workspaceRoot,
            env: env
        });

        return this.docsTerminal;
    }

    async selectFeatures(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const availableFeatures = this.cargoWorkspace.getAvailableFeatures();
        const selectedFeatures = this.cargoWorkspace.selectedFeatures;

        if (availableFeatures.length === 0) {
            vscode.window.showInformationMessage('No features available for current package selection');
            return;
        }

        // Convert features to QuickPickItems with checkbox state
        const items: vscode.QuickPickItem[] = availableFeatures.map(feature => ({
            label: feature,
            description: selectedFeatures.has(feature) ? '✓ Selected' : '',
            detail: feature === 'all-features' ? 'Enable all features' : `Feature: ${feature}`
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select features (click to toggle)',
            canPickMany: false
        });

        if (selected) {
            // Toggle the selected feature
            this.cargoWorkspace.toggleFeature(selected.label);
        }
    }

    async toggleFeature(feature: string): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        this.cargoWorkspace.toggleFeature(feature);
    }

    // Context menu command wrappers for specific targets

    async setBuildTarget(target: CargoTarget): Promise<void> {
        await this.setTarget(TargetActionType.Build, target);
    }

    async setRunTarget(target: CargoTarget): Promise<void> {
        await this.setTarget(TargetActionType.Run, target);
    }

    async setTestTarget(target: CargoTarget): Promise<void> {
        await this.setTarget(TargetActionType.Test, target);
    }

    async setBenchTarget(target: CargoTarget): Promise<void> {
        await this.setTarget(TargetActionType.Bench, target);
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
    async setTarget(actionType: TargetActionType, target: CargoTarget | null): Promise<void> {
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

        if (!this.cargoWorkspace || !this.taskProvider) {
            throw new Error('No cargo workspace or task provider available');
        }

        // Debug action requires special handling
        if (actionType === TargetActionType.Debug) {
            await this.startDebugSession(target);
            return;
        }

        // Use the task provider to create and execute a VS Code task for other actions
        const task = this.taskProvider.createTaskForTargetAction(target, actionType);
        if (task) {
            await vscode.tasks.executeTask(task);
            vscode.window.showInformationMessage(`Running ${actionType} for ${target.name}...`);
        } else {
            throw new Error(`Failed to create task for ${actionType} on ${target.name}`);
        }
    }

    /**
     * Start a debug session for a specific target
     */
    private async startDebugSession(target: CargoTarget): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        // First, build the target to ensure we have debug symbols
        await this.buildTargetForDebug(target);

        // Get the executable path
        const executablePath = this.getExecutablePathForTarget(target);

        // Create debug configuration
        const debugConfig: vscode.DebugConfiguration = {
            name: `Debug ${target.name}`,
            type: 'lldb', // Default to LLDB, could be configurable
            request: 'launch',
            program: executablePath,
            args: [], // Could be made configurable
            cwd: this.getWorkingDirectoryForTarget(target),
            stopOnEntry: false,
            showDisplayString: true,
            sourceLanguages: ['rust']
        };

        // Start the debug session
        const started = await vscode.debug.startDebugging(
            vscode.workspace.getWorkspaceFolder(vscode.Uri.file(this.cargoWorkspace.workspaceRoot)),
            debugConfig
        );

        if (started) {
            vscode.window.showInformationMessage(`Started debugging ${target.name}...`);
        } else {
            throw new Error('Failed to start debug session');
        }
    }

    /**
     * Build target with debug symbols
     */
    private async buildTargetForDebug(target: CargoTarget): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider) {
            throw new Error('No cargo workspace or task provider available');
        }

        // Create build task for debug
        const task = this.taskProvider.createTaskForTargetAction(target, TargetActionType.Build);
        if (task) {
            // Execute build and wait for completion
            const execution = await vscode.tasks.executeTask(task);

            // Wait for the task to complete
            await new Promise<void>((resolve, reject) => {
                const disposable = vscode.tasks.onDidEndTask(e => {
                    if (e.execution === execution) {
                        disposable.dispose();
                        if (e.execution.task.definition.type === 'cargo') {
                            resolve();
                        } else {
                            reject(new Error('Build task failed'));
                        }
                    }
                });
            });
        } else {
            throw new Error('Failed to create build task for debug');
        }
    }

    /**
     * Get the executable path for a target
     */
    private getExecutablePathForTarget(target: CargoTarget): string {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        // Get target directory from metadata, fallback to default
        const targetDir = this.cargoWorkspace.targetDirectory ||
            path.join(this.cargoWorkspace.workspaceRoot, 'target');

        // Build path components: target_dir[/platform_target]/profile/executable
        const pathComponents = [targetDir];

        // Add platform target if explicitly set
        if (this.cargoWorkspace.selectedPlatformTarget) {
            pathComponents.push(this.cargoWorkspace.selectedPlatformTarget);
        }

        // Add profile directory - default to 'dev' when no explicit selection
        const profile = this.cargoWorkspace.currentProfile.toString();
        let profilePath: string;
        if (profile === 'none') {
            profilePath = 'debug'; // Default to dev profile (debug directory) when no explicit selection
        } else if (profile === 'dev') {
            profilePath = 'debug';
        } else {
            profilePath = profile;
        }
        pathComponents.push(profilePath);

        // Handle different target types and their subdirectories
        if (target.kind.includes('example')) {
            // Examples are placed in the examples/ subdirectory
            pathComponents.push('examples');
        }

        let executableName = target.name;

        // Handle different target types
        if (target.kind.includes('bin')) {
            executableName = target.name;
        } else if (target.kind.includes('example')) {
            executableName = target.name;
        }

        // On Windows, add .exe extension
        if (process.platform === 'win32') {
            executableName += '.exe';
        }

        pathComponents.push(executableName);

        return path.join(...pathComponents);
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
                await this.setTarget(actionType, selected.target);
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

        // Build environment variables from configuration
        const env = this.buildCargoEnvironment(command);

        const terminal = vscode.window.createTerminal({
            name: `Cargo ${command}`,
            cwd: this.cargoWorkspace.workspaceRoot,
            env: env
        });

        const cargoCommand = this.workspaceConfig.cargoCommand || 'cargo';

        // Split cargoCommand at whitespaces - first part is command, rest are additional args
        const commandParts = cargoCommand.trim().split(/\s+/);
        const actualCommand = commandParts[0];
        const cargoCommandArgs = commandParts.slice(1);

        const args = this.cargoWorkspace.getCargoArgs(command);

        // Add extra arguments based on command type
        const extraArgs = this.getExtraArgsForCommand(command);

        // Combine all arguments: cargoCommand args + base args + extra args
        const allArgs = [...cargoCommandArgs, ...args, ...extraArgs];

        const commandLine = `${actualCommand} ${allArgs.join(' ')}`;

        terminal.sendText(commandLine);
        terminal.show();
    }

    /**
     * Build environment variables for cargo command execution
     */
    private buildCargoEnvironment(command: string): { [key: string]: string } {
        const env: { [key: string]: string } = {};

        // Start with base extraEnv
        if (this.workspaceConfig.extraEnv) {
            Object.assign(env, this.workspaceConfig.extraEnv);
        }

        // Add legacy environment settings for backward compatibility
        if (this.workspaceConfig.environment) {
            Object.assign(env, this.workspaceConfig.environment);
        }

        // Add command-specific environment variables
        if (command === 'run' || command === 'bench') {
            // For run and bench commands, merge run.extraEnv
            const runExtraEnv = this.workspaceConfig.runExtraEnv || {};
            Object.assign(env, runExtraEnv);
        } else if (command === 'test') {
            // For test commands, merge test.extraEnv
            const testExtraEnv = this.workspaceConfig.testExtraEnv || {};
            Object.assign(env, testExtraEnv);
        }

        return env;
    }

    /**
     * Get extra arguments for specific command types
     */
    private getExtraArgsForCommand(command: string): string[] {
        if (command === 'run' || command === 'bench') {
            return this.workspaceConfig.runExtraArgs || [];
        } else if (command === 'test') {
            return this.workspaceConfig.testExtraArgs || [];
        }
        return [];
    }

    // Command wrappers for new action-based commands

    async executeDefaultBuild(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Build);
    }

    async executeDefaultRun(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Run);
    }

    async executeDefaultDebug(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Debug);
    }

    async executeDefaultTest(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Test);
    }

    async executeDefaultBench(): Promise<void> {
        await this.executeDefaultAction(TargetActionType.Bench);
    }

    async executeBuildAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Build);
    }

    async executeRunAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Run);
    }

    async executeDebugAction(target: CargoTarget): Promise<void> {
        await this.executeTargetAction(target, TargetActionType.Debug);
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
        // Also respect the selected package setting - only add package arg when specific package is selected
        const workingDirectory = this.getWorkingDirectoryForTarget(target);
        const isExecutingFromPackageDir = target.packagePath && workingDirectory === target.packagePath;
        const selectedPackage = this.cargoWorkspace!.selectedPackage;
        const shouldIncludePackageArg = selectedPackage &&
            target.packageName && this.cargoWorkspace!.isWorkspace && !isExecutingFromPackageDir;

        if (shouldIncludePackageArg) {
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

    // ==================== PROJECT OUTLINE COMMANDS ====================

    /**
     * Select a package from the Project Outline view
     */
    async projectOutline_selectPackage(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !node?.data?.memberName) {
            return;
        }

        await this.cargoWorkspace.setSelectedPackage(node.data.memberName);
        console.log(`Selected package: ${node.data.memberName}`);
    }

    /**
     * Unselect the current package (set to "No selection")
     */
    async projectOutline_unselectPackage(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        await this.cargoWorkspace.setSelectedPackage(undefined);
        console.log('Unselected package (set to All)');
    }

    /**
     * Set a target as the build target from Project Outline
     */
    async projectOutline_setBuildTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        // Set package selection if different
        if (target.packageName !== this.cargoWorkspace.selectedPackage) {
            await this.cargoWorkspace.setSelectedPackage(target.packageName);
        }

        // Set build target - for libraries, just store "lib"
        if (target.kind.includes('lib')) {
            this.cargoWorkspace.setSelectedBuildTarget('lib');
        } else {
            this.cargoWorkspace.setSelectedBuildTarget(target.name);
        }

        console.log(`Set build target: ${target.name} in package: ${target.packageName}`);
    }

    /**
     * Unset the current build target
     */
    async projectOutline_unsetBuildTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        this.cargoWorkspace.setSelectedBuildTarget(null);
        console.log('Unset build target');
    }

    /**
     * Set a target as the run target from Project Outline
     */
    async projectOutline_setRunTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        // Set package selection if different
        if (target.packageName !== this.cargoWorkspace.selectedPackage) {
            await this.cargoWorkspace.setSelectedPackage(target.packageName);
        }

        this.cargoWorkspace.setSelectedRunTarget(target.name);
        console.log(`Set run target: ${target.name} in package: ${target.packageName}`);
    }

    /**
     * Unset the current run target
     */
    async projectOutline_unsetRunTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        this.cargoWorkspace.setSelectedRunTarget(null);
        console.log('Unset run target');
    }

    /**
     * Set a target as the benchmark target from Project Outline
     */
    async projectOutline_setBenchmarkTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        // Set package selection if different
        if (target.packageName !== this.cargoWorkspace.selectedPackage) {
            await this.cargoWorkspace.setSelectedPackage(target.packageName);
        }

        this.cargoWorkspace.setSelectedBenchmarkTarget(target.name);
        console.log(`Set benchmark target: ${target.name} in package: ${target.packageName}`);
    }

    /**
     * Unset the current benchmark target
     */
    async projectOutline_unsetBenchmarkTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        this.cargoWorkspace.setSelectedBenchmarkTarget(null);
        console.log('Unset benchmark target');
    }

    /**
     * Toggle a feature from Project Outline
     */
    async projectOutline_toggleFeature(feature: string): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Use the workspace's toggleFeature method which has proper mutual exclusion logic
        this.cargoWorkspace.toggleFeature(feature);
        console.log(`Toggled feature: ${feature}`);
    }

    // ==================== PROJECT OUTLINE ACTION COMMANDS ====================

    /**
     * Build a package directly without changing current selections
     */
    async projectOutline_buildPackage(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider || !node?.data?.memberName) {
            return;
        }

        const packageName = node.data.memberName;

        try {
            // Create task definition for package build
            const taskDefinition: any = {
                type: 'cargo',
                command: 'build',
                packageName: packageName
            };

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                `Build ${packageName}`,
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);
                vscode.window.showInformationMessage(`Building package ${packageName}...`);
            } else {
                throw new Error('Failed to create build task');
            }
        } catch (error) {
            console.error('Package build failed:', error);
            vscode.window.showErrorMessage(`Build failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Test a package directly without changing current selections
     */
    async projectOutline_testPackage(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider || !node?.data?.memberName) {
            return;
        }

        const packageName = node.data.memberName;

        try {
            // Create task definition for package test
            const taskDefinition: any = {
                type: 'cargo',
                command: 'test',
                packageName: packageName
            };

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                `Test ${packageName}`,
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);
                vscode.window.showInformationMessage(`Testing package ${packageName}...`);
            } else {
                throw new Error('Failed to create test task');
            }
        } catch (error) {
            console.error('Package test failed:', error);
            vscode.window.showErrorMessage(`Test failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Build a specific target directly without changing current selections
     */
    async projectOutline_buildTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        try {
            // Create task definition for target build
            const taskDefinition: any = {
                type: 'cargo',
                command: 'build',
                packageName: target.packageName
            };

            // Add target-specific arguments based on target kind
            if (target.kind.includes('lib')) {
                taskDefinition.targetKind = 'lib';
            } else if (target.kind.includes('bin')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'bin';
            } else if (target.kind.includes('example')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'example';
            } else if (target.kind.includes('test')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'test';
            } else if (target.kind.includes('bench')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'bench';
            }

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                `Build ${target.name}`,
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);
                vscode.window.showInformationMessage(`Building target ${target.name}...`);
            } else {
                throw new Error('Failed to create build task');
            }
        } catch (error) {
            console.error('Target build failed:', error);
            vscode.window.showErrorMessage(`Build failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Run a specific target directly without changing current selections
     */
    async projectOutline_runTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        try {
            // Create task definition for target run
            const taskDefinition: any = {
                type: 'cargo',
                command: 'run',
                packageName: target.packageName
            };

            // Add target-specific arguments based on target kind
            if (target.kind.includes('bin')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'bin';
            } else if (target.kind.includes('example')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'example';
            } else {
                throw new Error(`Target ${target.name} is not runnable`);
            }

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                `Run ${target.name}`,
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);
                vscode.window.showInformationMessage(`Running target ${target.name}...`);
            } else {
                throw new Error('Failed to create run task');
            }
        } catch (error) {
            console.error('Target run failed:', error);
            vscode.window.showErrorMessage(`Run failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Debug a specific target directly without changing current selections
     */
    async projectOutline_debugTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        // Check if target is debuggable
        if (!target.supportsActionType(TargetActionType.Debug)) {
            vscode.window.showErrorMessage(`Target ${target.name} is not debuggable`);
            return;
        }

        try {
            await this.startDebugSession(target);
        } catch (error) {
            console.error('Target debug failed:', error);
            vscode.window.showErrorMessage(`Debug failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Run benchmark for a specific target directly without changing current selections
     */
    async projectOutline_benchTarget(node?: any): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider || !node?.data) {
            return;
        }

        const target = node.data as CargoTarget;

        try {
            // Create task definition for target benchmark
            const taskDefinition: any = {
                type: 'cargo',
                command: 'bench',
                packageName: target.packageName
            };

            // Add target-specific arguments
            if (target.kind.includes('bench')) {
                taskDefinition.targetName = target.name;
                taskDefinition.targetKind = 'bench';
            } else {
                throw new Error(`Target ${target.name} is not a benchmark target`);
            }

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                `Benchmark ${target.name}`,
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);
                vscode.window.showInformationMessage(`Running benchmark ${target.name}...`);
            } else {
                throw new Error('Failed to create benchmark task');
            }
        } catch (error) {
            console.error('Target benchmark failed:', error);
            vscode.window.showErrorMessage(`Benchmark failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    // ==================== PROJECT OUTLINE FILTER COMMANDS ====================

    /**
     * Set workspace member filter
     */
    async projectOutline_setWorkspaceMemberFilter(): Promise<void> {
        if (!this.projectOutlineTreeProvider) {
            return;
        }

        await this.projectOutlineTreeProvider.setWorkspaceMemberFilter();
    }

    /**
     * Clear workspace member filter
     */
    async projectOutline_clearWorkspaceMemberFilter(): Promise<void> {
        if (!this.projectOutlineTreeProvider) {
            return;
        }

        this.projectOutlineTreeProvider.clearWorkspaceMemberFilter();
    }

    /**
     * Show target type filter menu
     */
    async projectOutline_showTargetTypeFilter(): Promise<void> {
        if (!this.projectOutlineTreeProvider) {
            return;
        }

        await this.projectOutlineTreeProvider.showTargetTypeFilter();
    }

    /**
     * Clear target type filter
     */
    async projectOutline_clearTargetTypeFilter(): Promise<void> {
        if (!this.projectOutlineTreeProvider) {
            return;
        }

        this.projectOutlineTreeProvider.clearTargetTypeFilter();
    }

    /**
     * Clear all active filters
     */
    async projectOutline_clearAllFilters(): Promise<void> {
        if (!this.projectOutlineTreeProvider) {
            return;
        }

        this.projectOutlineTreeProvider.clearAllFilters();
    }

    /**
     * Toggle workspace member grouping in Project Outline view
     */
    async projectOutline_toggleWorkspaceMemberGrouping(): Promise<void> {
        if (!this.projectOutlineTreeProvider) {
            return;
        }

        this.projectOutlineTreeProvider.toggleWorkspaceMemberGrouping();
    }

    // ==================== PROJECT STATUS COMMANDS ====================

    /**
     * Build command from Project Status view - executes current build selection
     */
    async projectStatus_build(): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider) {
            throw new Error('No cargo workspace or task provider available');
        }

        try {
            const selectedBuildTarget = this.cargoWorkspace.selectedBuildTarget;
            const selectedPackage = this.cargoWorkspace.selectedPackage;

            // Create task definition based on current selection
            const taskDefinition: any = {
                type: 'cargo',
                command: 'build'
            };

            // Add package if selected
            if (selectedPackage && this.cargoWorkspace.isWorkspace) {
                taskDefinition.packageName = selectedPackage;
            }

            // Handle different target types
            if (selectedBuildTarget) {
                if (selectedBuildTarget === 'lib') {
                    // Library target - just set the kind, don't override package selection
                    taskDefinition.targetKind = 'lib';
                } else {
                    // Find the actual target to get its kind
                    const target = this.cargoWorkspace.targets.find(t => t.name === selectedBuildTarget);
                    if (target) {
                        taskDefinition.targetName = target.name;
                        taskDefinition.targetKind = Array.isArray(target.kind) ? target.kind[0] : target.kind;
                        // Only override package if no package was explicitly selected
                        if (!selectedPackage || !this.cargoWorkspace.isWorkspace) {
                            taskDefinition.packageName = target.packageName;
                        }
                    } else {
                        throw new Error(`Target ${selectedBuildTarget} not found`);
                    }
                }
            }
            // If selectedBuildTarget is null, we build all targets (no target-specific args)

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Add features
            const selectedFeatures = Array.from(this.cargoWorkspace.selectedFeatures);
            if (selectedFeatures.includes('all-features')) {
                taskDefinition.allFeatures = true;
            } else if (selectedFeatures.length > 0) {
                const regularFeatures = selectedFeatures.filter(f => f !== 'all-features');
                if (regularFeatures.length > 0) {
                    taskDefinition.features = regularFeatures;
                }
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                'Build',
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);

                // Show appropriate message
                let message = 'Building';
                if (selectedBuildTarget) {
                    message += ` ${selectedBuildTarget}`;
                } else {
                    message += ' all targets';
                }
                if (selectedPackage) {
                    message += ` for ${selectedPackage}`;
                }
                message += '...';

                vscode.window.showInformationMessage(message);
            } else {
                throw new Error('Failed to create build task');
            }
        } catch (error) {
            console.error('Build command failed:', error);
            vscode.window.showErrorMessage(`Build failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Run command from Project Status view - executes current run selection
     */
    async projectStatus_run(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        try {
            const selectedRunTarget = this.cargoWorkspace.selectedRunTarget;
            const selectedPackage = this.cargoWorkspace.selectedPackage;

            if (!selectedPackage) {
                throw new Error('Cannot run targets when no package is selected. Please select a specific package.');
            }

            if (selectedRunTarget) {
                // Specific target selected - find and execute it
                const target = this.cargoWorkspace.targets.find(t => t.name === selectedRunTarget);
                if (target) {
                    await this.executeTargetAction(target, TargetActionType.Run);
                } else {
                    throw new Error(`Target ${selectedRunTarget} not found`);
                }
            } else {
                // Auto-detect runnable target for the selected package
                const packageTargets = this.cargoWorkspace.targets.filter(t => t.packageName === selectedPackage);
                const runnableTarget = packageTargets.find(t =>
                    Array.isArray(t.kind) ?
                        t.kind.includes('bin') || t.kind.includes('example') :
                        t.kind === 'bin' || t.kind === 'example'
                );

                if (runnableTarget) {
                    await this.executeTargetAction(runnableTarget, TargetActionType.Run);
                } else {
                    throw new Error(`No runnable targets found in package ${selectedPackage}`);
                }
            }
        } catch (error) {
            console.error('Run command failed:', error);
            vscode.window.showErrorMessage(`Run failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Debug command from Project Status view - debugs current run selection
     */
    async projectStatus_debug(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        try {
            const selectedRunTarget = this.cargoWorkspace.selectedRunTarget;
            const selectedPackage = this.cargoWorkspace.selectedPackage;

            if (!selectedPackage) {
                throw new Error('Cannot debug targets when no package is selected. Please select a specific package.');
            }

            if (selectedRunTarget) {
                // Specific target selected - find and debug it
                const target = this.cargoWorkspace.targets.find(t => t.name === selectedRunTarget);
                if (target) {
                    await this.executeTargetAction(target, TargetActionType.Debug);
                } else {
                    throw new Error(`Target ${selectedRunTarget} not found`);
                }
            } else {
                // Auto-detect debuggable target for the selected package
                const packageTargets = this.cargoWorkspace.targets.filter(t => t.packageName === selectedPackage);
                const debuggableTarget = packageTargets.find(t =>
                    Array.isArray(t.kind) ?
                        t.kind.includes('bin') || t.kind.includes('example') :
                        t.kind === 'bin' || t.kind === 'example'
                );

                if (debuggableTarget) {
                    await this.executeTargetAction(debuggableTarget, TargetActionType.Debug);
                } else {
                    throw new Error(`No debuggable targets found in package ${selectedPackage}`);
                }
            }
        } catch (error) {
            console.error('Debug command failed:', error);
            vscode.window.showErrorMessage(`Debug failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Test command from Project Status view - executes tests for current selection
     */
    async projectStatus_test(): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider) {
            throw new Error('No cargo workspace or task provider available');
        }

        try {
            const selectedPackage = this.cargoWorkspace.selectedPackage;

            // Create task definition for tests
            const taskDefinition: any = {
                type: 'cargo',
                command: 'test'
            };

            // Add package if selected
            if (selectedPackage && this.cargoWorkspace.isWorkspace) {
                taskDefinition.packageName = selectedPackage;
            }

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Add features
            const selectedFeatures = Array.from(this.cargoWorkspace.selectedFeatures);
            if (selectedFeatures.includes('all-features')) {
                taskDefinition.allFeatures = true;
            } else if (selectedFeatures.length > 0) {
                const regularFeatures = selectedFeatures.filter(f => f !== 'all-features');
                if (regularFeatures.length > 0) {
                    taskDefinition.features = regularFeatures;
                }
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                'Test',
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);

                const message = selectedPackage ? `Running tests for ${selectedPackage}...` : 'Running tests for all packages...';
                vscode.window.showInformationMessage(message);
            } else {
                throw new Error('Failed to create test task');
            }
        } catch (error) {
            console.error('Test command failed:', error);
            vscode.window.showErrorMessage(`Test failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Benchmark command from Project Status view - executes current benchmark selection
     */
    async projectStatus_bench(): Promise<void> {
        if (!this.cargoWorkspace || !this.taskProvider) {
            throw new Error('No cargo workspace or task provider available');
        }

        try {
            const selectedBenchmarkTarget = this.cargoWorkspace.selectedBenchmarkTarget;
            const selectedPackage = this.cargoWorkspace.selectedPackage;

            // Create task definition
            const taskDefinition: any = {
                type: 'cargo',
                command: 'bench'
            };

            // Add package if selected
            if (selectedPackage && this.cargoWorkspace.isWorkspace) {
                taskDefinition.packageName = selectedPackage;
            }

            // Handle specific benchmark target
            if (selectedBenchmarkTarget) {
                // Find the actual target to get its details
                const target = this.cargoWorkspace.targets.find(t => t.name === selectedBenchmarkTarget);
                if (target) {
                    taskDefinition.targetName = target.name;
                    taskDefinition.targetKind = 'bench';
                    taskDefinition.packageName = target.packageName;
                } else {
                    throw new Error(`Benchmark target ${selectedBenchmarkTarget} not found`);
                }
            }
            // If no specific target, we bench all benchmarks (no target-specific args)

            // Add current profile
            if (this.cargoWorkspace.currentProfile.toString() === 'release') {
                taskDefinition.profile = 'release';
            }

            // Add features
            const selectedFeatures = Array.from(this.cargoWorkspace.selectedFeatures);
            if (selectedFeatures.includes('all-features')) {
                taskDefinition.allFeatures = true;
            } else if (selectedFeatures.length > 0) {
                const regularFeatures = selectedFeatures.filter(f => f !== 'all-features');
                if (regularFeatures.length > 0) {
                    taskDefinition.features = regularFeatures;
                }
            }

            // Create and execute the task
            const task = this.taskProvider.resolveTask(new vscode.Task(
                taskDefinition,
                vscode.TaskScope.Workspace,
                'Benchmark',
                'cargo'
            ));

            if (task) {
                await vscode.tasks.executeTask(task);

                let message = 'Running benchmarks';
                if (selectedBenchmarkTarget) {
                    message += ` for ${selectedBenchmarkTarget}`;
                } else if (selectedPackage) {
                    message += ` for ${selectedPackage}`;
                } else {
                    message += ' for all packages';
                }
                message += '...';

                vscode.window.showInformationMessage(message);
            } else {
                throw new Error('Failed to create benchmark task');
            }
        } catch (error) {
            console.error('Benchmark command failed:', error);
            vscode.window.showErrorMessage(`Benchmark failed: ${error instanceof Error ? error.message : String(error)}`);
        }
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

        // Dispose docs terminal if it exists
        if (this.docsTerminal) {
            try {
                this.docsTerminal.dispose();
            } catch (error) {
                console.error('Error disposing docs terminal:', error);
            }
            this.docsTerminal = undefined;
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

    private getTargetsForPackage(packageName: string): CargoTarget[] {
        if (!this.cargoWorkspace) {
            return [];
        }

        return this.cargoWorkspace.targets.filter(target => target.packageName === packageName);
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
