import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { VsCodeTask } from './wasm/cargo_tools_vscode';

export async function echo_task(msg: string) {
    const task = new vscode.Task(
        { type: 'shell' },
        vscode.TaskScope.Workspace,
        'echo',
        'echo',
        new vscode.ShellExecution("echo", [msg])
    );

    await vscode.tasks.executeTask(task);
    vscode.window.showInformationMessage(`Running echo cmd...`);
}

function spawnWithOutput(
    cmd: string,
    args: string[]
): Promise<{ stdout: string; stderr: string }> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        throw new Error('No workspace folder found');
    }

    return new Promise((resolve, reject) => {
        const child = spawn(cmd, args, { cwd: workspaceFolder.uri.fsPath });

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

export async function execute_async(command: string, rest: string[]): Promise<String> {
    const { stdout } = await spawnWithOutput(command, rest);
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
