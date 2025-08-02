import * as vscode from 'vscode';
import { CargoWorkspace } from './cargoWorkspace';

export interface CargoTaskDefinition extends vscode.TaskDefinition {
    command: string;
    profile?: string;
    target?: string;
    features?: string[];
    allFeatures?: boolean;
    noDefaultFeatures?: boolean;
}

export class CargoTaskProvider implements vscode.TaskProvider {
    static CargoType = 'cargo';

    constructor(private workspace: CargoWorkspace) { }

    public provideTasks(): Thenable<vscode.Task[]> {
        return this.getCargoTasks();
    }

    public resolveTask(task: vscode.Task): vscode.Task | undefined {
        const definition = task.definition as CargoTaskDefinition;
        if (definition.type === CargoTaskProvider.CargoType && definition.command) {
            return this.createCargoTask(definition);
        }
        return undefined;
    }

    private async getCargoTasks(): Promise<vscode.Task[]> {
        const tasks: vscode.Task[] = [];
        const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');

        // Common cargo commands
        const commands = ['build', 'run', 'test', 'check', 'clean', 'doc'];

        for (const command of commands) {
            // Create task for current configuration
            const currentTask = this.createCargoTask({
                type: CargoTaskProvider.CargoType,
                command: command
            });
            tasks.push(currentTask);

            // Create additional variants for different profiles
            if (command === 'build' || command === 'run' || command === 'test') {
                const releaseTask = this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: command,
                    profile: 'release'
                });
                tasks.push(releaseTask);
            }
        }

        // Add target-specific tasks
        for (const target of this.workspace.targets) {
            if (target.isExecutable) {
                const runTask = this.createCargoTask({
                    type: CargoTaskProvider.CargoType,
                    command: 'run',
                    target: target.name
                });
                tasks.push(runTask);
            }
        }

        return tasks;
    }

    private createCargoTask(definition: CargoTaskDefinition): vscode.Task {
        const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');
        const args = this.buildCargoArgs(definition);

        const execution = new vscode.ShellExecution(cargoPath, args, {
            cwd: this.workspace.workspaceRoot
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

    private buildCargoArgs(definition: CargoTaskDefinition): string[] {
        const args = [definition.command];

        // Add profile
        if (definition.profile === 'release' ||
            (this.workspace.currentProfile.toString() === 'release' && !definition.profile)) {
            args.push('--release');
        }

        // Add target
        if (definition.target) {
            const target = this.workspace.targets.find(t => t.name === definition.target);
            if (target) {
                if (target.isExecutable) {
                    args.push('--bin', target.name);
                } else if (target.isLibrary) {
                    args.push('--lib');
                }
            }
        } else if (this.workspace.currentTarget && definition.command !== 'clean') {
            const target = this.workspace.currentTarget;
            if (target.isExecutable) {
                args.push('--bin', target.name);
            } else if (target.isLibrary) {
                args.push('--lib');
            }
        }

        // Add features
        if (definition.features && definition.features.length > 0) {
            args.push('--features', definition.features.join(','));
        }

        if (definition.allFeatures) {
            args.push('--all-features');
        }

        if (definition.noDefaultFeatures) {
            args.push('--no-default-features');
        }

        // Add configuration-based arguments
        const config = vscode.workspace.getConfiguration('cargoTools');
        const commandArgs = config.get<string[]>(`${definition.command}Args`, []);
        args.push(...commandArgs);

        return args;
    }

    private getTaskName(definition: CargoTaskDefinition): string {
        let name = `cargo ${definition.command}`;

        if (definition.target) {
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
                return ['$rustc'];
            default:
                return [];
        }
    }
}
