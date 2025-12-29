import * as vscode from 'vscode';
import { ConfigPropertyType } from './wasm/cargo_tools_vscode';

export function get_config(section: string, key: string, type: ConfigPropertyType, default_value: any): any {
    let dir = vscode.workspace.workspaceFolders?.[0]?.uri;
    let config = vscode.workspace.getConfiguration(section, dir);

    switch (type) {
        case ConfigPropertyType.String:
            return config.get<string>(key, default_value as string);
        case ConfigPropertyType.Boolean:
            return config.get<boolean>(key, default_value as boolean);
        case ConfigPropertyType.Array:
            return config.get<string[]>(key, default_value as string[]);
        case ConfigPropertyType.Object:
            return config.get<{ [key: string]: string }>(key, default_value as { [key: string]: string });
    }
}

