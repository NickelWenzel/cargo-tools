import { exec } from 'child_process';
import { promisify } from 'util';
import * as vscode from 'vscode';

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

export async function execute_async(command: string): Promise<String> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        throw new Error('No workspace folder found');
    }

    const { stdout } = await promisify(exec)(command, {
        cwd: workspaceFolder.uri.fsPath,
        timeout: 10000 // 10 second timeout
    });
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