import * as vscode from 'vscode';
import { ProjectStatusTreeProvider } from './projectStatusTreeProvider';
import { ProjectOutlineTreeProvider } from './projectOutlineTreeProvider';
import { CargoExtensionManager } from './cargoExtensionManager';

let extensionManager: CargoExtensionManager | undefined;

/**
 * Main extension activation function.
 * @param context The extension context
 * @returns A promise that will resolve when the extension is ready for use
 */
export async function activate(context: vscode.ExtensionContext): Promise<any> {
	try {
		console.log('Cargo Tools extension activation started...');

		// Initialize and start the extension manager
		extensionManager = await CargoExtensionManager.create(context);

		console.log('Cargo Tools extension fully initialized!');
		return setup(context);
	} catch (error) {
		console.error('Failed to activate Cargo Tools extension:', error);
		vscode.window.showErrorMessage(`Cargo Tools extension failed to activate: ${error}`);
		throw error;
	}
}

/**
 * Setup function that configures the extension components and commands
 */
async function setup(context: vscode.ExtensionContext): Promise<any> {
	if (!extensionManager) {
		throw new Error('Extension manager not initialized');
	}

	// Wait for the extension manager to be fully ready
	// This handles the case where workspace detection is asynchronous
	await extensionManager.waitForInitialization();

	// If we don't have a cargo workspace, exit early with minimal activation
	if (!extensionManager.hasCargoProject()) {
		console.log('No Cargo workspace found - extension activated with minimal features');
		await setCargoContext(false);
		return {};
	}

	// We have a cargo project, set the context
	await setCargoContext(true);

	// Get the workspace from the extension manager
	const cargoWorkspace = extensionManager.getCargoWorkspace();
	if (!cargoWorkspace) {
		console.log('No cargo workspace available');
		return {};
	}

	// New tree providers for the modern UI
	const projectStatusProvider = new ProjectStatusTreeProvider();
	const projectOutlineProvider = new ProjectOutlineTreeProvider();

	// Update providers with workspace
	projectStatusProvider.updateWorkspace(cargoWorkspace);
	projectOutlineProvider.updateWorkspace(cargoWorkspace);

	// Register new tree views
	vscode.window.createTreeView('cargoToolsProjectStatus', {
		treeDataProvider: projectStatusProvider,
		showCollapseAll: false,
		canSelectMany: false
	});

	vscode.window.createTreeView('cargoToolsProjectOutline', {
		treeDataProvider: projectOutlineProvider,
		showCollapseAll: true,
		canSelectMany: false
	});

	// Subscribe to workspace changes to update providers
	cargoWorkspace.onDidChangeTargets(() => {
		projectStatusProvider.refresh();
		projectOutlineProvider.refresh();
	});

	cargoWorkspace.onDidChangeProfile(() => {
		projectStatusProvider.refresh();
	});

	cargoWorkspace.onDidChangeTarget(() => {
		projectStatusProvider.refresh();
		projectOutlineProvider.refresh();
	});

	// Task provider is registered by the extension manager, so we don't need to do it here

	return {
		extensionManager,
		workspace: cargoWorkspace,
		projectStatusProvider,
		projectOutlineProvider
	};
}

/**
 * Helper function to set cargo context for when clauses
 */
async function setCargoContext(hasCargo: boolean): Promise<void> {
	await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasCargo', hasCargo);
	console.log(`Context set: cargoTools:workspaceHasCargo = ${hasCargo}`);
}





/**
 * This method is called when the extension is deactivated.
 * Following CMake Tools deactivation pattern for proper cleanup.
 */
export async function deactivate(): Promise<void> {
	console.log('Deactivating Cargo Tools extension...');

	try {
		// Dispose extension manager first (this handles commands and workspace)
		if (extensionManager) {
			await extensionManager.asyncDispose();
			extensionManager = undefined;
		}

		console.log('Cargo Tools extension deactivated successfully');
	} catch (error) {
		console.error('Error during Cargo Tools extension deactivation:', error);
	}
}
