import * as vscode from 'vscode';
import { CargoExtensionManager } from './cargoExtensionManager';

/**
 * Variable substitution system for Cargo Tools extension.
 * Provides command substitutions that can be used in VS Code settings,
 * particularly for configuring Rust Analyzer based on current Cargo Tools selections.
 * 
 * Based on patterns from microsoft/vscode-cmake-tools extension.
 */

export interface CargoVariableContext {
    workspaceFolder: string;
    selectedPackage: string;
    selectedProfile: string;
    selectedBuildTarget: string;
    selectedRunTarget: string;
    selectedBenchmarkTarget: string;
    selectedPlatformTarget: string;
    selectedFeatures: string[];
    allFeatures: boolean;
    noDefaultFeatures: boolean;
}

/**
 * Command substitution provider for Cargo Tools extension.
 * Registers VS Code commands that can be used for variable substitution
 * in settings.json, particularly for Rust Analyzer configuration.
 */
export class CargoCommandSubstitution {
    private static readonly COMMAND_PREFIX = 'cargo-tools.get';

    constructor(
        private readonly extensionManager: CargoExtensionManager,
        private readonly context: vscode.ExtensionContext
    ) {
        this.registerCommands();
    }

    private registerCommands(): void {
        const commands = [
            // Basic selection commands
            { command: 'getSelectedPackage', handler: this.getSelectedPackage.bind(this) },
            { command: 'getSelectedProfile', handler: this.getSelectedProfile.bind(this) },
            { command: 'getSelectedBuildTarget', handler: this.getSelectedBuildTarget.bind(this) },
            { command: 'getSelectedRunTarget', handler: this.getSelectedRunTarget.bind(this) },
            { command: 'getSelectedBenchmarkTarget', handler: this.getSelectedBenchmarkTarget.bind(this) },
            { command: 'getSelectedPlatformTarget', handler: this.getSelectedPlatformTarget.bind(this) },

            // Feature-related commands
            { command: 'getSelectedFeatures', handler: this.getSelectedFeatures.bind(this) },
            { command: 'getSelectedFeaturesArray', handler: this.getSelectedFeaturesArray.bind(this) },
            { command: 'getAllFeatures', handler: this.getAllFeatures.bind(this) },
            { command: 'getNoDefaultFeatures', handler: this.getNoDefaultFeatures.bind(this) },

            // Cargo argument commands for Rust Analyzer integration
            { command: 'getCargoFeatureArgs', handler: this.getCargoFeatureArgs.bind(this) },
            { command: 'getCargoTargetArgs', handler: this.getCargoTargetArgs.bind(this) },
            { command: 'getCargoProfileArgs', handler: this.getCargoProfileArgs.bind(this) },
            { command: 'getCargoPackageArgs', handler: this.getCargoPackageArgs.bind(this) },
            { command: 'getCargoExtraArgs', handler: this.getCargoExtraArgs.bind(this) },

            // Combined argument commands
            { command: 'getCargoCheckArgs', handler: this.getCargoCheckArgs.bind(this) },
            { command: 'getCargoBuildArgs', handler: this.getCargoBuildArgs.bind(this) },
            { command: 'getCargoTestArgs', handler: this.getCargoTestArgs.bind(this) },
        ];

        for (const cmd of commands) {
            const fullCommand = `${CargoCommandSubstitution.COMMAND_PREFIX}${cmd.command}`;
            const disposable = vscode.commands.registerCommand(fullCommand, cmd.handler);
            this.context.subscriptions.push(disposable);
        }
    }

    private getVariableContext(): CargoVariableContext {
        const workspace = this.extensionManager.getCargoWorkspace();
        const workspaceFolder = workspace?.workspaceRoot || '';
        const config = vscode.workspace.getConfiguration('cargoTools');

        return {
            workspaceFolder,
            selectedPackage: workspace?.selectedPackage || 'All',
            selectedProfile: workspace?.currentProfile?.toString() || 'dev',
            selectedBuildTarget: workspace?.selectedBuildTarget || 'All',
            selectedRunTarget: workspace?.selectedRunTarget || '',
            selectedBenchmarkTarget: workspace?.selectedBenchmarkTarget || 'All',
            selectedPlatformTarget: workspace?.selectedPlatformTarget || '',
            selectedFeatures: workspace ? Array.from(workspace.selectedFeatures) : [],
            allFeatures: config.get<boolean>('allFeatures', false),
            noDefaultFeatures: config.get<boolean>('noDefaultFeatures', false)
        };
    }

    // Basic selection commands
    private getSelectedPackage(): string {
        return this.getVariableContext().selectedPackage;
    }

    private getSelectedProfile(): string {
        return this.getVariableContext().selectedProfile;
    }

    private getSelectedBuildTarget(): string {
        return this.getVariableContext().selectedBuildTarget;
    }

    private getSelectedRunTarget(): string {
        return this.getVariableContext().selectedRunTarget;
    }

    private getSelectedBenchmarkTarget(): string {
        return this.getVariableContext().selectedBenchmarkTarget;
    }

    private getSelectedPlatformTarget(): string {
        return this.getVariableContext().selectedPlatformTarget;
    }

    // Feature-related commands
    private getSelectedFeatures(): string {
        const context = this.getVariableContext();
        if (context.allFeatures) {
            return 'all';
        }
        return context.selectedFeatures.join(',');
    }

    private getSelectedFeaturesArray(): string[] {
        const context = this.getVariableContext();
        if (context.allFeatures) {
            return ['all'];
        }
        return context.selectedFeatures;
    }

    private getAllFeatures(): boolean {
        return this.getVariableContext().allFeatures;
    }

    private getNoDefaultFeatures(): boolean {
        return this.getVariableContext().noDefaultFeatures;
    }

    // Cargo argument commands for Rust Analyzer integration
    private getCargoFeatureArgs(): string[] {
        const context = this.getVariableContext();
        const args: string[] = [];

        if (context.allFeatures) {
            args.push('--all-features');
        } else if (context.selectedFeatures.length > 0) {
            args.push('--features', context.selectedFeatures.join(','));
        }

        if (context.noDefaultFeatures) {
            args.push('--no-default-features');
        }

        return args;
    }

    private getCargoTargetArgs(): string[] {
        const context = this.getVariableContext();
        const args: string[] = [];

        if (context.selectedPlatformTarget) {
            args.push('--target', context.selectedPlatformTarget);
        }

        return args;
    }

    private getCargoProfileArgs(): string[] {
        const context = this.getVariableContext();
        const args: string[] = [];

        if (context.selectedProfile && context.selectedProfile !== 'dev') {
            if (context.selectedProfile === 'release') {
                args.push('--release');
            } else {
                args.push('--profile', context.selectedProfile);
            }
        }

        return args;
    }

    private getCargoPackageArgs(): string[] {
        const context = this.getVariableContext();
        const args: string[] = [];

        if (context.selectedPackage && context.selectedPackage !== 'All') {
            args.push('-p', context.selectedPackage);
        }

        return args;
    }

    private getCargoExtraArgs(): string[] {
        // Combine all the common cargo arguments
        return [
            ...this.getCargoPackageArgs(),
            ...this.getCargoProfileArgs(),
            ...this.getCargoTargetArgs(),
            ...this.getCargoFeatureArgs()
        ];
    }

    // Combined argument commands for specific cargo operations
    private getCargoCheckArgs(): string[] {
        const baseArgs = this.getCargoExtraArgs();
        // Add check-specific arguments if needed
        return baseArgs;
    }

    private getCargoBuildArgs(): string[] {
        const baseArgs = this.getCargoExtraArgs();
        // Add build-specific arguments if needed  
        return baseArgs;
    }

    private getCargoTestArgs(): string[] {
        const baseArgs = this.getCargoExtraArgs();
        // Add test-specific arguments if needed
        return baseArgs;
    }

    /**
     * Dispose of all registered commands
     */
    dispose(): void {
        // Commands are disposed automatically through context.subscriptions
    }
}

/**
 * Initialize command substitution support for the Cargo Tools extension
 */
export function initializeCommandSubstitution(
    extensionManager: CargoExtensionManager,
    context: vscode.ExtensionContext
): CargoCommandSubstitution {
    return new CargoCommandSubstitution(extensionManager, context);
}
