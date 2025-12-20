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

export function get_state(key: string): any {
    const context = ensureContext();
    return context.workspaceState.get(key);
}

export async function set_state(key: string, value: any): Promise<void> {
    const context = ensureContext();
    await context.workspaceState.update(key, value);
}

export function get_configuration(): any {
    const config = vscode.workspace.getConfiguration('cargoTools');
    const configObj: Record<string, any> = {
        title: 'Cargo Tools',
        properties: {}
    };

    for (const key of Object.keys(config)) {
        if (key !== 'get' && key !== 'has' && key !== 'inspect' && key !== 'update') {
            const value = config.get(key);
            if (value !== undefined) {
                configObj.properties[key] = {
                    type: typeof value,
                    default: value,
                    description: ''
                };
            }
        }
    }

    return configObj;
}
