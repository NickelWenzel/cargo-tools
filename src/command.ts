import * as vscode from 'vscode';

let commands: vscode.Disposable[] = [];

export function register_command(command: string, callback: (...args: any[]) => any) {
    commands.push(vscode.commands.registerCommand(command, callback));
}

export function dispose_commands() {
    commands.forEach(cmd => {
        try {
            cmd.dispose();
        } catch (error) {
            console.error('Error disposing command:', error);
        }
    });

}