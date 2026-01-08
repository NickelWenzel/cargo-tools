import * as vscode from 'vscode';
import { on_current_dir_changed, on_file_changed } from './wasm/cargo_tools_vscode';
import { log } from 'console';

let nextHandle = 1;
const disposables = new Map<number, vscode.Disposable>();

export async function read_file(file_path: string): Promise<string> {
    const uri = vscode.Uri.file(file_path);
    const fileContent = await vscode.workspace.fs.readFile(uri);
    return new TextDecoder().decode(fileContent);
}

export function watch_current_dir(): number {
    const handle = nextHandle++;

    const disposable = vscode.workspace.onDidChangeWorkspaceFolders(async () => {
        const dir = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? '';
        await on_current_dir_changed(dir);
    });

    disposables.set(handle, disposable);

    return handle;
}

export function unwatch_current_dir(handle: number): void {
    const disposable = disposables.get(handle);
    if (disposable) {
        disposable.dispose();
        disposables.delete(handle);
    }
}

export function watch_file(path: string): number {
    console.log(`Watch ${path}`);
    const handle = nextHandle++;

    const watcher = vscode.workspace.createFileSystemWatcher(path);

    const changeDisposable = watcher.onDidChange(async () => {
        console.log(`Changed ${path}`);
        await on_file_changed(path);
    });

    const createDisposable = watcher.onDidCreate(async () => {
        console.log(`Created ${path}`);
        await on_file_changed(path);
    });

    const deleteDisposable = watcher.onDidDelete(async () => {
        console.log(`Deleted ${path}`);
        await on_file_changed(path);
    });

    const compositeDisposable = {
        dispose: () => {
            changeDisposable.dispose();
            createDisposable.dispose();
            deleteDisposable.dispose();
            watcher.dispose();
        }
    };

    disposables.set(handle, compositeDisposable);

    return handle;
}

export function unwatch_file(handle: number): void {
    const disposable = disposables.get(handle);
    if (disposable) {
        disposable.dispose();
        disposables.delete(handle);
    }
}
