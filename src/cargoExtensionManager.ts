import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTaskProvider } from './cargoTaskProvider';
import { StatusBarProvider } from './statusBarProvider';
import { ProfilesTreeProvider } from './profilesTreeProvider';
import { TargetsTreeProvider } from './targetsTreeProvider';
import { WorkspaceTreeProvider } from './workspaceTreeProvider';
import { CargoProfile } from './cargoProfile';
import { CargoTarget } from './cargoTarget';

/**
 * The main extension manager that coordinates all cargo-tools functionality.
 * This is the singleton that manages the extension lifecycle and state.
 */
export class CargoExtensionManager implements vscode.Disposable {
    private static instance?: CargoExtensionManager;

    // Core components
    private cargoWorkspace?: CargoWorkspace;
    private taskProvider?: CargoTaskProvider;
    private statusBarProvider?: StatusBarProvider;
    private profilesTreeProvider?: ProfilesTreeProvider;
    private targetsTreeProvider?: TargetsTreeProvider;
    private workspaceTreeProvider?: WorkspaceTreeProvider;

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
            await CargoExtensionManager.instance.init();
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
     * Initialize the extension manager and all components
     */
    private async init(): Promise<void> {
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
        this.taskProvider = new CargoTaskProvider(this.cargoWorkspace);
        const taskProviderDisposable = vscode.tasks.registerTaskProvider('cargo', this.taskProvider);
        this.subscriptions.push(taskProviderDisposable);
    }

    /**
     * Register all extension commands with error handling wrapper
     */
    private registerCommands(): void {
        const commands = [
            'build',
            'run',
            'test',
            'debug',
            'clean',
            'selectProfile',
            'selectTarget',
            'refresh'
        ] as const;

        for (const command of commands) {
            const disposable = this.registerCommand(command);
            this.subscriptions.push(disposable);
        }
    }

    /**
     * Register a single command with error handling wrapper
     */
    private registerCommand<K extends keyof CargoExtensionManager>(name: K): vscode.Disposable {
        return vscode.commands.registerCommand(`cargo-tools.${name}`, async (...args: any[]) => {
            const correlationId = this.generateCorrelationId();
            try {
                console.log(`[${correlationId}] cargo-tools.${name} started`);

                if (!this.cargoWorkspace) {
                    vscode.window.showErrorMessage('No cargo workspace found. Please open a Rust project.');
                    return;
                }

                const method = this[name] as Function;
                if (typeof method !== 'function') {
                    throw new Error(`Command ${name} is not implemented`);
                }

                const result = await method.call(this, ...args);
                console.log(`[${correlationId}] cargo-tools.${name} completed`);
                return result;
            } catch (error) {
                console.error(`[${correlationId}] cargo-tools.${name} failed:`, error);
                vscode.window.showErrorMessage(`Cargo Tools: ${name} failed - ${error}`);
                throw error;
            }
        });
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

        // Subscribe to workspace events using the correct event names
        const targetChangedSub = this.cargoWorkspace.onDidChangeTarget((target: CargoTarget | null) => {
            this.targetsTreeProvider?.refresh();
        });

        const profileChangedSub = this.cargoWorkspace.onDidChangeProfile((profile: CargoProfile) => {
            this.profilesTreeProvider?.refresh();
        });

        const targetsChangedSub = this.cargoWorkspace.onDidChangeTargets((targets: CargoTarget[]) => {
            this.targetsTreeProvider?.refresh();
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
            return;
        }

        const target = this.cargoWorkspace.currentTarget;
        const profile = this.cargoWorkspace.currentProfile;

        // Use existing command for now - will be improved in task provider enhancements
        await vscode.commands.executeCommand('cargo-tools.build');
    }

    async run(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const target = this.cargoWorkspace.currentTarget;
        if (!target || !target.isExecutable) {
            vscode.window.showErrorMessage('Selected target is not executable');
            return;
        }

        // Use existing command for now
        await vscode.commands.executeCommand('cargo-tools.run');
    }

    async test(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Use existing command for now
        await vscode.commands.executeCommand('cargo-tools.test');
    }

    async debug(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        const target = this.cargoWorkspace.currentTarget;
        if (!target || !target.isExecutable) {
            vscode.window.showErrorMessage('Selected target is not executable');
            return;
        }

        // Use existing command for now
        await vscode.commands.executeCommand('cargo-tools.debug');
    }

    async clean(): Promise<void> {
        if (!this.cargoWorkspace) {
            return;
        }

        // Use existing command for now
        await vscode.commands.executeCommand('cargo-tools.clean');
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

    dispose(): void {
        this.disposeWorkspaceSubscriptions();
        this.subscriptions.forEach(sub => sub.dispose());
        // CargoWorkspace doesn't have dispose method in current implementation
        CargoExtensionManager.instance = undefined;
    }
}
