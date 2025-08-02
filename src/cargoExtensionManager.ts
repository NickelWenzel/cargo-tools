import * as vscode from 'vscode';
import * as path from 'path';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTaskProvider } from './cargoTaskProvider';
import { StatusBarProvider } from './statusBarProvider';
import { ProfilesTreeProvider } from './profilesTreeProvider';
import { TargetsTreeProvider } from './targetsTreeProvider';
import { WorkspaceTreeProvider } from './workspaceTreeProvider';
import { CargoProfile } from './cargoProfile';
import { CargoTarget } from './cargoTarget';
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
    private statusBarProvider?: StatusBarProvider;
    private profilesTreeProvider?: ProfilesTreeProvider;
    private targetsTreeProvider?: TargetsTreeProvider;
    private workspaceTreeProvider?: WorkspaceTreeProvider;

    // Configuration management
    private readonly workspaceConfig: CargoConfigurationReader = CargoConfigurationReader.create();

    // Subscriptions for cleanup
    private subscriptions: vscode.Disposable[] = [];
    private workspaceSubscriptions: vscode.Disposable[] = [];

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

        // Initialize status bar with workspace
        this.statusBarProvider = new StatusBarProvider(this.cargoWorkspace);
        this.subscriptions.push(this.statusBarProvider);

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
        // Register command with improved CMake Tools-style wrapper
        function register<K extends keyof CargoExtensionManager>(name: K) {
            return vscode.commands.registerCommand(`cargo-tools.${name}`, async (...args: any[]) => {
                // Generate a unique ID that can be correlated in the log file
                const correlationId = generateCorrelationId();

                try {
                    console.log(`[${correlationId}] cargo-tools.${name} started`);

                    const command = (CargoExtensionManager.instance![name] as Function).bind(CargoExtensionManager.instance);
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
        }

        // List of commands to register - matches CMake Tools pattern
        const commands: (keyof CargoExtensionManager)[] = [
            'build',
            'run',
            'test',
            'debug',
            'clean',
            'selectProfile',
            'selectTarget',
            'refresh'
        ];

        // Register all commands
        for (const command of commands) {
            const disposable = register(command);
            this.subscriptions.push(disposable);
        }
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
    async build(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const target = this.cargoWorkspace.currentTarget;
        const profile = this.cargoWorkspace.currentProfile;
        
        await this.executeCargoCommand('build');
    }

    async run(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const target = this.cargoWorkspace.currentTarget;
        if (!target || !target.isExecutable) {
            throw new Error('Selected target is not executable');
        }

        await this.executeCargoCommand('run');
    }

    async test(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        await this.executeCargoCommand('test');
    }

    async debug(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const target = this.cargoWorkspace.currentTarget;
        if (!target || !target.isExecutable) {
            throw new Error('Selected target is not executable');
        }

        // Build first
        await this.executeCargoCommand('build');

        // Start debugging session
        const debugConfig = {
            name: 'Debug Rust',
            type: 'cppdbg',
            request: 'launch',
            program: this.getTargetExecutablePath(),
            cwd: this.cargoWorkspace.workspaceRoot,
            args: [],
            stopAtEntry: false,
            environment: [],
            console: 'integratedTerminal'
        };

        await vscode.debug.startDebugging(undefined, debugConfig);
    }

    async clean(): Promise<void> {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        await this.executeCargoCommand('clean');
    }

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

    async selectTarget(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const targets = this.cargoWorkspace.targets;
        const items = targets.map(target => ({
            label: target.name,
            description: `${target.kind} in ${target.packageName || 'main'}`,
            target: target
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a target'
        });

        if (selected) {
            await this.cargoWorkspace.setTarget(selected.target);
        }
    }

    async refresh(): Promise<void> {
        if (this.cargoWorkspace) {
            await this.cargoWorkspace.initialize();
            vscode.window.showInformationMessage('Cargo workspace refreshed');
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

    /**
     * Get the executable path for the current target
     */
    private getTargetExecutablePath(): string {
        if (!this.cargoWorkspace) {
            throw new Error('No cargo workspace available');
        }

        const target = this.cargoWorkspace.currentTarget;
        if (!target) {
            throw new Error('No target selected');
        }

        const profile = this.cargoWorkspace.currentProfile === CargoProfile.release ? 'release' : 'debug';
        return path.join(this.cargoWorkspace.workspaceRoot, 'target', profile, target.name);
    }

    dispose(): void {
        this.disposeWorkspaceSubscriptions();
        this.subscriptions.forEach(sub => sub.dispose());
        this.workspaceConfig.dispose();
        // CargoWorkspace doesn't have dispose method in current implementation
        CargoExtensionManager.instance = undefined;
    }
}
