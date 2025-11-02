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