import * as vscode from 'vscode';
import { run, ExitToken } from './wasm/cargo_tools_vscode';
import { initializeStateModule } from './context';

/**
 * Global extension context - accessible throughout the extension
 */
export let extension_context: vscode.ExtensionContext | undefined;

let exit: ExitToken | undefined;

/**
 * Main extension activation function.
 * @param context The extension context
 * @returns A promise that will resolve when the extension is ready for use
 */
export async function activate(context: vscode.ExtensionContext): Promise<any> {
	extension_context = context;
	try {
		initializeStateModule(context);
		console.log('Cargo Tools extension activation started...');

		// Initialize and start the extension manager
		// extensionManager = await CargoExtensionManager.create(context);

		const workspaceFolder = vscode.workspace.workspaceFolders?.[0].uri.fsPath;
		if (!workspaceFolder) {
			throw new Error('No workspace folder found');
		}

		exit = run(workspaceFolder);

		console.log('Cargo Tools extension fully initialized!');
		return {};
	} catch (error) {
		await setCargoContext(false);
		console.error('Failed to activate Cargo Tools extension:', error);
		vscode.window.showErrorMessage(`Cargo Tools extension failed to activate: ${error}`);
		throw error;
	}
}

/**
 * Helper function to set cargo context for when clauses
 */
async function setCargoContext(hasCargo: boolean): Promise<void> {
	await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasCargo', hasCargo);
	console.log(`Context set: cargoTools:workspaceHasCargo = ${hasCargo}`);
}

/**
 * Helper function to set makefile context for when clauses
 */
async function setMakefileContext(hasMakefile: boolean): Promise<void> {
	await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasMakefile', hasMakefile);
	console.log(`Context set: cargoTools:workspaceHasMakefile = ${hasMakefile}`);
}

/**
 * This method is called when the extension is deactivated.
 */
export async function deactivate(): Promise<void> {
	console.log('Deactivating Cargo Tools extension...');

	try {
		exit?.exit();

		console.log('Cargo Tools extension deactivated successfully');
	} catch (error) {
		console.error('Error during Cargo Tools extension deactivation:', error);
	}
}
