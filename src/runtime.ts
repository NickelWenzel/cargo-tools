import * as vscode from 'vscode';
import { extension_context, log } from './extension';

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

export function log_debug(msg: string) {
    return log.debug(msg);
}

export function log_info(msg: string) {
    return log.info(msg);
}

export function log_warn(msg: string) {
    return log.warn(msg);
}

export function log_error(msg: string) {
    return log.error(msg);
}

export function register_command(command: string, callback: (args: any[]) => any) {
    extension_context?.subscriptions.push(vscode.commands.registerCommand(command, (...args: any[]) => { return callback([...args]); }));
}

export function get_state(key: string): string | undefined {
    return extension_context?.workspaceState.get(key);
}

export async function set_state(key: string, value: string): Promise<void> {
    await extension_context?.workspaceState.update(key, value);
}