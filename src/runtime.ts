import * as vscode from 'vscode';
import { on_current_dir_changed, on_file_changed } from './wasm/cargo_tools_vscode';

let nextHandle = 1;
const disposables = new Map<number, vscode.Disposable>();

export function watch_current_dir(): number {
    const handle = nextHandle++;

    const disposable = vscode.workspace.onDidChangeWorkspaceFolders(() => {
        const dir = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? '';
        on_current_dir_changed(dir);
    });

    disposables.set(handle, disposable);

    const initialDir = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? '';
    if (initialDir) {
        on_current_dir_changed(initialDir);
    }

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
    const handle = nextHandle++;

    const watcher = vscode.workspace.createFileSystemWatcher(path);

    const changeDisposable = watcher.onDidChange(() => {
        on_file_changed(path);
    });

    const createDisposable = watcher.onDidCreate(() => {
        on_file_changed(path);
    });

    const deleteDisposable = watcher.onDidDelete(() => {
        on_file_changed(path);
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
