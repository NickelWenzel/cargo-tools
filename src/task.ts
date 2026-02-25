import * as vscode from 'vscode';
import { VsCodeTask } from './wasm/cargo_tools_vscode';

export async function execute_task(cargo_tools_task: VsCodeTask): Promise<void> {
    const cmd = cargo_tools_task.cmd();
    const args = cargo_tools_task.args();

    const definition: vscode.TaskDefinition = {
        type: cargo_tools_task.task_type(),
        args: args,
    };

    const execution = new vscode.ShellExecution(cmd, args);

    const task = new vscode.Task(
        definition,
        vscode.TaskScope.Workspace,
        `${cmd} ${args.join(" ")}`,
        definition.type,
        execution,
        ['$rustc']
    );

    task.presentationOptions = {
        echo: true,
        reveal: vscode.TaskRevealKind.Always,
        focus: false,
        panel: vscode.TaskPanelKind.Shared,
        showReuseMessage: true,
        clear: false
    };

    try {
        await vscode.tasks.executeTask(task);
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        vscode.window.showErrorMessage(`Failed to run cargo make task: ${message}`);
    }
}