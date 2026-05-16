import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { VsCodeTask, VsCodeProcess } from './wasm/cargo_tools_vscode';

function spawnWithOutput(cargo_tools_process: VsCodeProcess): Promise<{ stdout: string; stderr: string }> {
    const cmd = cargo_tools_process.cmd();
    const args = cargo_tools_process.args();
    const env: { [key: string]: string } = Object.fromEntries(cargo_tools_process.env());

    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        throw new Error('No workspace folder found');
    }

    return new Promise((resolve, reject) => {
        const child = spawn(cmd, args, { cwd: workspaceFolder.uri.fsPath, env: { ...process.env, ...env } });

        let stdout = "";
        let stderr = "";

        child.stdout.setEncoding("utf8");
        child.stderr.setEncoding("utf8");

        child.stdout.on("data", d => (stdout += d));
        child.stderr.on("data", d => (stderr += d));

        child.on("error", reject);
        child.on("close", code => {
            code === 0
                ? resolve({ stdout, stderr })
                : reject(new Error(stderr || `exit ${code}`));
        });
    });
}

export async function execute_async(cargo_tools_process: VsCodeProcess): Promise<String> {
    const { stdout } = await spawnWithOutput(cargo_tools_process);
    return stdout;
}

export async function executeCommand(command: string, rest: any[]): Promise<any> {
    return await vscode.commands.executeCommand(command, ...rest);
}

export async function showInformationMessage(message: string, items: string[]): Promise<string | undefined> {
    return await vscode.window.showInformationMessage(message, ...items);
}

export async function showErrorMessage(message: string, items: string[]): Promise<string | undefined> {
    return await vscode.window.showErrorMessage(message, ...items);
}

export async function execute_task(cargo_tools_task: VsCodeTask): Promise<void> {
    const cmd = cargo_tools_task.cmd();
    const args = cargo_tools_task.args();
    const env: { [key: string]: string } = Object.fromEntries(cargo_tools_task.env());

    const definition: vscode.TaskDefinition = {
        type: cargo_tools_task.task_type(),
        args: args,
    };

    const execution = new vscode.ShellExecution(cmd, args, { env });

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
