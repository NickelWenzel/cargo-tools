import * as vscode from 'vscode';

let extensionContext: vscode.ExtensionContext | undefined;

export function initializeStateModule(context: vscode.ExtensionContext): void {
    extensionContext = context;
}

function ensureContext(): vscode.ExtensionContext {
    if (!extensionContext) {
        throw new Error('State module not initialized. Call initializeStateModule first.');
    }
    return extensionContext;
}

export function get_state(key: string): string | undefined {
    const context = ensureContext();
    return context.workspaceState.get(key);
}

export async function set_state(key: string, value: string): Promise<void> {
    const context = ensureContext();
    await context.workspaceState.update(key, value);
}
