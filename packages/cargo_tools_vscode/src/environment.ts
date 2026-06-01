import * as vscode from 'vscode';
import { ConfigValueType } from '../../../vscode_extension/src/wasm/cargo_tools_vscode';

export function get_config(section: string, key: string, type: ConfigValueType, default_value: any): any {
    let dir = vscode.workspace.workspaceFolders?.[0]?.uri;
    let config = vscode.workspace.getConfiguration(section, dir);
    switch (type) {
        case ConfigValueType.String:
            return config.get<string>(key, default_value as string);
        case ConfigValueType.Boolean:
            return config.get<boolean>(key, default_value as boolean);
        case ConfigValueType.VecString:
            return config.get<string[]>(key, default_value as string[]);
        case ConfigValueType.HashMapString:
            return config.get<{ [key: string]: string }>(key, default_value as { [key: string]: string });
    }
}
