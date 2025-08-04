import * as vscode from 'vscode';
import { CargoExtensionManager } from './cargoExtensionManager';

let extensionManager: CargoExtensionManager | undefined;

export async function activate(context: vscode.ExtensionContext) {
    try {
        console.log('Cargo Tools extension activation started...');

        // Initialize the extension manager
        extensionManager = await CargoExtensionManager.create(context);

        console.log('Cargo Tools extension activated successfully');

        // Return the extension manager instance for external API access
        return {
            getExtensionManager: () => extensionManager
        };
    } catch (error) {
        console.error('Failed to activate Cargo Tools extension:', error);
        vscode.window.showErrorMessage(`Failed to activate Cargo Tools: ${error}`);
        throw error;
    }
}

export function deactivate() {
    console.log('Cargo Tools extension deactivation started...');

    if (extensionManager) {
        extensionManager.dispose();
        extensionManager = undefined;
    }

    console.log('Cargo Tools extension deactivated');
}

// Export for testing and external access
export function getExtensionManager(): CargoExtensionManager | undefined {
    return extensionManager;
}
