import * as vscode from 'vscode';
import { extension_context } from '../../../../vscode_extension/src/extension';

export function register_command(command: string, callback: (args: any[]) => any) {
    extension_context?.subscriptions.push(vscode.commands.registerCommand(command, (...args: any[]) => { return callback([...args]); }));
}
