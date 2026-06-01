import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { VsCodeTask, VsCodeProcess } from '../../../vscode_extension/src/wasm/cargo_tools_vscode';
import { extension_context } from '../../../vscode_extension/src/extension';

export class FileWatcher {
    private watcher?: vscode.Disposable;
    private onChanged?: (() => void);

    on_changed(callback: () => void): void {
        this.onChanged = callback;
    }

    watch_files(paths: string[]): void {
        // Create a single watcher with a pattern that matches all paths
        const pattern = new vscode.RelativePattern(
            vscode.workspace.workspaceFolders?.[0]?.uri ?? '',
            `{${paths.map((path) => vscode.workspace.asRelativePath(path)).join(',')}}`
        );

        const watcher = vscode.workspace.createFileSystemWatcher(pattern);
        const changeDisposable = watcher.onDidChange(() => this.onChanged?.());
        const createDisposable = watcher.onDidCreate(() => this.onChanged?.());
        const deleteDisposable = watcher.onDidDelete(() => this.onChanged?.());

        const compositeDisposable = {
            dispose: () => {
                changeDisposable.dispose();
                createDisposable.dispose();
                deleteDisposable.dispose();
                watcher.dispose();
            }
        };

        this.watcher?.dispose();
        this.watcher = compositeDisposable;
    }

    dispose(): void {
        this.watcher?.dispose();
    }
}

export async function read_file(file_path: string): Promise<string> {
    const uri = vscode.Uri.file(file_path);
    const fileContent = await vscode.workspace.fs.readFile(uri);
    return new TextDecoder().decode(fileContent);
}

export async function debug(target_exe_path: string, target_name: string): Promise<void> {
    // Create debug configuration
    const debugConfig: vscode.DebugConfiguration = {
        name: `Debug ${target_name}`,
        type: 'lldb', // Default to LLDB, could be configurable
        request: 'launch',
        program: target_exe_path,
        args: [], // Could be made configurable
        stopOnEntry: false,
        showDisplayString: true,
        sourceLanguages: ['rust']
    };

    // Start the debug session
    const started = await vscode.debug.startDebugging(undefined, debugConfig);

    if (started) {
        vscode.window.showInformationMessage(`Started debugging ${target_name}...`);
    } else {
        throw new Error(`Failed to start debug session for ${target_name}`);
    }
}

export function host_platform(): string {
    return process.platform;
}

export function get_state(key: string): string | undefined {
    return extension_context?.workspaceState.get(key);
}

export async function set_state(key: string, value: string): Promise<void> {
    await extension_context?.workspaceState.update(key, value);
}

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
