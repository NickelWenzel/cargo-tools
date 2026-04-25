import * as vscode from 'vscode';

export function get_config(section: string, key: string, default_value: string): any {
    let dir = vscode.workspace.workspaceFolders?.[0]?.uri;
    let config = vscode.workspace.getConfiguration(section, dir);
    return config.get<string>(key, default_value);
}

