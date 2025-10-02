import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoTarget, CargoTargetKind, TargetActionType } from './cargoTarget';
import { CargoConfigurationReader } from './cargoConfigurationReader';

export interface CargoTaskDefinition extends vscode.TaskDefinition {
    command: string;
    args?: string[]; // Additional arguments for the command
    profile?: string;
    target?: string;
    targetKind?: CargoTargetKind;
    features?: string[];
    allFeatures?: boolean;
}

export interface CargoMakeTaskDefinition extends vscode.TaskDefinition {
    task: string; // The cargo-make task name
}

export class CargoTaskProvider implements vscode.TaskProvider {
    static CargoType = 'cargo';

    constructor(
        private workspace: CargoWorkspace,
        private configReader?: CargoConfigurationReader
    ) { }

    private isWorkspace(): boolean {
        // Check if we have multiple packages (indicating a workspace)
        const packageNames = new Set(this.workspace.targets.map(t => t.packageName).filter(Boolean));
        return packageNames.size > 1;
    }

    public provideTasks(): Thenable<vscode.Task[]> {
        return this.getCargoTasks();
    }

    public resolveTask(task: vscode.Task): vscode.Task | undefined {
        const definition = task.definition;
        if (definition.type === CargoTaskProvider.CargoType && definition.command) {
            return this.createCargoTask(definition as CargoTaskDefinition);
        } else if (definition.type === 'cargo-make' && definition.task) {
            return this.createCargoMakeTask(definition as CargoMakeTaskDefinition);
        }
        return undefined;
    }

    private async getCargoTasks(): Promise<vscode.Task[]> {
        const tasks: vscode.Task[] = [];
        // Use configuration reader if available, otherwise fall back to direct VS Code config
        const cargoCommand = this.configReader
            ? this.configReader.cargoCommand
            : vscode.workspace.getConfiguration('cargoTools').get<string>('cargoCommand', 'cargo');

        // Common cargo commands (without specific targets)
        const baseCommands = ['build', 'check', 'clean', 'doc'];

        for (const command of baseCommands) {
            // Create task for current configuration
            const currentTask = this.createCargoTask({
                type: CargoTaskProvider.CargoType,
                command: command
            });
            tasks.push(currentTask);

            // Create additional variants for different profiles
            if (command === 'build' || command === 'check') {
                const releaseTask = this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: command,
                    profile: 'release'
                });
                tasks.push(releaseTask);
            }
        }

        // Add target-specific tasks based on target types
        for (const target of this.workspace.targets) {
            this.addTargetSpecificTasks(tasks, target);
        }

        // Add general test and run commands
        tasks.push(this.createCargoTask({
            type: CargoTaskProvider.CargoType,
            command: 'test'
        }));

        tasks.push(this.createCargoTask({
            type: CargoTaskProvider.CargoType,
            command: 'test',
            profile: 'release'
        }));

        // Add workspace-level run task (runs default binary)
        const defaultBinary = this.workspace.targets.find(t => t.isExecutable);
        if (defaultBinary) {
            tasks.push(this.createCargoTask({
                type: CargoTaskProvider.CargoType,
                command: 'run'
            }));

            tasks.push(this.createCargoTask({
                type: CargoTaskProvider.CargoType,
                command: 'run',
                profile: 'release'
            }));
        }

        return tasks;
    }

    private addTargetSpecificTasks(tasks: vscode.Task[], target: CargoTarget): void {
        const targetKind = target.kind; // Primary kind

        switch (targetKind) {
            case CargoTargetKind.Bin:
                // Binary targets: build and run
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Bin
                }));

                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'run',
                    target: target.name,
                    targetKind: CargoTargetKind.Bin
                }));

                // Release variants
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Bin,
                    profile: 'release'
                }));

                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'run',
                    target: target.name,
                    targetKind: CargoTargetKind.Bin,
                    profile: 'release'
                }));
                break;

            case CargoTargetKind.Example:
                // Example targets: build and run
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Example
                }));

                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'run',
                    target: target.name,
                    targetKind: CargoTargetKind.Example
                }));

                // Release variants
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'run',
                    target: target.name,
                    targetKind: CargoTargetKind.Example,
                    profile: 'release'
                }));
                break;

            case CargoTargetKind.Test:
                // Test targets: build and run
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Test
                }));

                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'test',
                    target: target.name,
                    targetKind: CargoTargetKind.Test
                }));
                break;

            case CargoTargetKind.Bench:
                // Benchmark targets: build and run
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Bench
                }));

                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'bench',
                    target: target.name,
                    targetKind: CargoTargetKind.Bench
                }));
                break;

            case CargoTargetKind.Lib:
                // Library targets: build and test
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Lib
                }));

                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'test',
                    target: target.name,
                    targetKind: CargoTargetKind.Lib
                }));

                // Release variant
                tasks.push(this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'build',
                    target: target.name,
                    targetKind: CargoTargetKind.Lib,
                    profile: 'release'
                }));
                break;
        }
    }

    private createCargoTask(definition: CargoTaskDefinition): vscode.Task {
        // Get command override settings from configuration
        const config = this.configReader || { runCommandOverride: '', testCommandOverride: '' };

        let command: string;
        let args: string[];

        // Check if we should use command overrides
        if (definition.command === 'run' && config.runCommandOverride && config.runCommandOverride.trim() !== '') {
            // Use run command override - split into command and initial args
            const overrideTokens = config.runCommandOverride.trim().split(/\s+/);
            command = overrideTokens[0];
            const overrideArgs = overrideTokens.slice(1);

            // Build cargo args but replace 'run' command with override args
            const cargoArgs = this.buildCargoArgs(definition);
            const runIndex = cargoArgs.indexOf('run');
            if (runIndex >= 0) {
                // Replace 'run' with override args, keep the rest
                args = [...overrideArgs, ...cargoArgs.slice(runIndex + 1)];
            } else {
                // Fallback: use override args + all cargo args
                args = [...overrideArgs, ...cargoArgs];
            }
        } else if (definition.command === 'test' && config.testCommandOverride && config.testCommandOverride.trim() !== '') {
            // Use test command override - split into command and initial args
            const overrideTokens = config.testCommandOverride.trim().split(/\s+/);
            command = overrideTokens[0];
            const overrideArgs = overrideTokens.slice(1);

            // Build cargo args but replace 'test' command with override args
            const cargoArgs = this.buildCargoArgs(definition);
            const testIndex = cargoArgs.indexOf('test');
            if (testIndex >= 0) {
                // Replace 'test' with override args, keep the rest
                args = [...overrideArgs, ...cargoArgs.slice(testIndex + 1)];
            } else {
                // Fallback: use override args + all cargo args
                args = [...overrideArgs, ...cargoArgs];
            }
        } else {
            // Use configured cargo command instead of hardcoded 'cargo'
            const cargoCommand = this.configReader?.cargoCommand ||
                vscode.workspace.getConfiguration('cargoTools').get<string>('cargoCommand', 'cargo');

            // Split cargoCommand at whitespaces - first part is command, rest are additional args
            const commandParts = cargoCommand.trim().split(/\s+/);
            command = commandParts[0];
            const cargoCommandArgs = commandParts.slice(1);

            // Build cargo args and prepend any additional command args
            const cargoArgs = this.buildCargoArgs(definition);
            args = [...cargoCommandArgs, ...cargoArgs];
        }

        // Build environment variables
        const env = this.buildEnvironment(definition);

        const execution = new vscode.ShellExecution(command, args, {
            cwd: this.workspace.workspaceRoot,
            env: env
        });

        const task = new vscode.Task(
            definition,
            vscode.TaskScope.Workspace,
            this.getTaskName(definition),
            CargoTaskProvider.CargoType,
            execution,
            this.getProblemMatchers(definition.command)
        );

        task.group = this.getTaskGroup(definition.command);
        task.presentationOptions = {
            echo: true,
            reveal: vscode.TaskRevealKind.Always,
            focus: false,
            panel: vscode.TaskPanelKind.Shared,
            showReuseMessage: true,
            clear: false
        };

        return task;
    }

    private createCargoMakeTask(definition: CargoMakeTaskDefinition): vscode.Task {
        // Use configured cargo command
        const cargoCommand = this.configReader?.cargoCommand ||
            vscode.workspace.getConfiguration('cargoTools').get<string>('cargoCommand', 'cargo');

        // Split cargoCommand at whitespaces - first part is command, rest are additional args
        const commandParts = cargoCommand.trim().split(/\s+/);
        const command = commandParts[0];
        const cargoCommandArgs = commandParts.slice(1);

        // Build args: [additional cargo args, 'make', task]
        const args = [...cargoCommandArgs, 'make', definition.task];

        const execution = new vscode.ShellExecution(command, args, {
            cwd: this.workspace.workspaceRoot
        });

        const task = new vscode.Task(
            definition,
            vscode.TaskScope.Workspace,
            `make ${definition.task}`,
            'cargo-make',
            execution,
            ['$rustc']
        );

        task.group = vscode.TaskGroup.Build;
        task.presentationOptions = {
            echo: true,
            reveal: vscode.TaskRevealKind.Always,
            focus: false,
            panel: vscode.TaskPanelKind.Shared,
            showReuseMessage: true,
            clear: false
        };

        return task;
    }

    private buildCargoArgs(definition: CargoTaskDefinition): string[] {
        const args = [definition.command];

        // Add any custom arguments for the command
        if (definition.args && definition.args.length > 0) {
            args.push(...definition.args);
        }

        // Add profile - handle both task-specific and workspace default profiles
        const profileToUse = definition.profile || this.workspace.currentProfile.toString();

        if (profileToUse !== 'none') {
            args.push('--profile', profileToUse);
        }

        // Find target to get package information
        let targetObj: CargoTarget | undefined;
        const targetName = definition.target || definition.targetName;
        if (targetName) {
            targetObj = this.workspace.targets.find(t => t.name === targetName);
        }
        // Don't fall back to workspace.currentTarget when no target is specified
        // This allows "All" targets builds without target-specific restrictions

        // Add package argument if we have package info and it's needed
        const packageName = definition.packageName || targetObj?.packageName;
        if (packageName && this.isWorkspace()) {
            args.push('--package', packageName);
        }

        // Add target-specific arguments
        const targetNameForArgs = definition.target || definition.targetName;
        if (definition.targetKind) {
            switch (definition.targetKind) {
                case 'bin':
                    if (targetNameForArgs) {
                        args.push('--bin', targetNameForArgs);
                    }
                    break;
                case 'lib':
                    args.push('--lib');
                    break;
                case 'example':
                    if (targetNameForArgs) {
                        args.push('--example', targetNameForArgs);
                    }
                    break;
                case 'test':
                    if (targetNameForArgs) {
                        args.push('--test', targetNameForArgs);
                    }
                    break;
                case 'bench':
                    if (targetNameForArgs) {
                        args.push('--bench', targetNameForArgs);
                    }
                    break;
            }
        } else if (targetNameForArgs) {
            // Fallback: try to find target in workspace and determine type
            const target = this.workspace.targets.find(t => t.name === targetNameForArgs);
            if (target) {
                if (target.isExecutable) {
                    args.push('--bin', target.name);
                } else if (target.isLibrary) {
                    args.push('--lib');
                } else if (target.isExample) {
                    args.push('--example', target.name);
                } else if (target.isTest) {
                    args.push('--test', target.name);
                } else if (target.isBench) {
                    args.push('--bench', target.name);
                }
            }
        }
        // No fallback to workspace.currentTarget - when no target is specified, 
        // we want to build all targets (no target-specific args)

        // Add features
        if (definition.features && definition.features.length > 0) {
            args.push('--features', definition.features.join(','));
        }

        if (definition.allFeatures) {
            args.push('--all-features');
        }

        // Add platform target if selected
        if (this.workspace.selectedPlatformTarget) {
            args.push('--target', this.workspace.selectedPlatformTarget);
        }

        // Add configuration-based arguments
        const config = vscode.workspace.getConfiguration('cargoTools');
        const commandArgs = config.get<string[]>(`${definition.command}Args`, []);
        args.push(...commandArgs);

        // Add extra arguments based on command type
        if (this.configReader) {
            if (definition.command === 'run' || definition.command === 'bench') {
                // For run and bench commands, append run.extraArgs
                const runExtraArgs = this.configReader.runExtraArgs || [];
                args.push(...runExtraArgs);
            } else if (definition.command === 'test') {
                // For test commands, append test.extraArgs
                const testExtraArgs = this.configReader.testExtraArgs || [];
                args.push(...testExtraArgs);
            }
        }

        return args;
    }

    /**
     * Build environment variables for task execution
     */
    private buildEnvironment(definition: CargoTaskDefinition): { [key: string]: string } {
        const env: { [key: string]: string } = {};

        // Start with base extraEnv
        if (this.configReader?.extraEnv) {
            Object.assign(env, this.configReader.extraEnv);
        }

        // Add command-specific environment variables
        if (this.configReader) {
            if (definition.command === 'run' || definition.command === 'bench') {
                // For run and bench commands, merge run.extraEnv
                const runExtraEnv = this.configReader.runExtraEnv || {};
                Object.assign(env, runExtraEnv);
            } else if (definition.command === 'test') {
                // For test commands, merge test.extraEnv
                const testExtraEnv = this.configReader.testExtraEnv || {};
                Object.assign(env, testExtraEnv);
            }
        }

        return env;
    }

    private getTaskName(definition: CargoTaskDefinition): string {
        let name = `cargo ${definition.command}`;

        if (definition.target && definition.targetKind) {
            name += ` ${definition.targetKind} (${definition.target})`;
        } else if (definition.target) {
            name += ` (${definition.target})`;
        }

        if (definition.profile) {
            name += ` [${definition.profile}]`;
        }

        return name;
    }

    private getTaskGroup(command: string): vscode.TaskGroup | undefined {
        switch (command) {
            case 'build':
            case 'check':
                return vscode.TaskGroup.Build;
            case 'test':
            case 'bench':
                return vscode.TaskGroup.Test;
            case 'clean':
                return vscode.TaskGroup.Clean;
            default:
                return undefined;
        }
    }

    private getProblemMatchers(command: string): string[] {
        switch (command) {
            case 'build':
            case 'check':
            case 'run':
            case 'test':
            case 'bench':
                return ['$rustc'];
            default:
                return [];
        }
    }

    /**
     * Create a VS Code task for a specific target action
     */
    public createTaskForTargetAction(target: CargoTarget, actionType: TargetActionType): vscode.Task | undefined {
        if (!target.supportsActionType(actionType)) {
            return undefined;
        }

        const command = target.getCargoCommand(actionType);
        const targetKind = target.kind;

        // Handle features properly
        const selectedFeatures = this.workspace?.selectedFeatures ? Array.from(this.workspace.selectedFeatures) : [];
        const hasAllFeatures = selectedFeatures.includes('all-features');
        const regularFeatures = selectedFeatures.filter(f => f !== 'all-features');

        // Only include package name if workspace has a specific package selected
        // When no package is selected (undefined), we want to build all packages
        const selectedPackage = this.workspace?.selectedPackage;
        const packageName = selectedPackage ? target.packageName : undefined;

        const definition: CargoTaskDefinition = {
            type: 'cargo',
            command: command,
            targetName: target.name,
            targetKind: targetKind,
            packageName: packageName,
            features: regularFeatures.length > 0 ? regularFeatures : undefined,
            allFeatures: hasAllFeatures
        };

        return this.createCargoTask(definition);
    }
}
