import * as vscode from 'vscode';

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

        this.watcher = compositeDisposable; const disposable = vscode.workspace.onDidChangeWorkspaceFolders(async () => {
        });
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