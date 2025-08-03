import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';
import { ProjectStatusTreeProvider } from './projectStatusTreeProvider';
import { ProjectOutlineTreeProvider } from './projectOutlineTreeProvider';
import { PinnedCommands } from './pinnedCommandsTreeProvider';
import { CargoTaskProvider } from './cargoTaskProvider';
import { CargoExtensionManager } from './cargoExtensionManager';

let extensionManager: CargoExtensionManager | undefined;
let cargoWorkspace: CargoWorkspace | undefined;

// Helper function to check if we're in a workspace
function isWorkspace(workspace: CargoWorkspace): boolean {
	const packageNames = new Set(workspace.targets.map(t => t.packageName).filter(Boolean));
	return packageNames.size > 1;
}

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
		await extensionManager.init();

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

	// Get the legacy components from the extension manager
	cargoWorkspace = extensionManager.getCargoWorkspace();
	if (!cargoWorkspace) {
		console.log('No cargo workspace available');
		return {};
	}

	// New tree providers for the modern UI
	const projectStatusProvider = new ProjectStatusTreeProvider();
	const projectOutlineProvider = new ProjectOutlineTreeProvider();
	const pinnedCommands = new PinnedCommands(context);

	// Update providers with workspace
	projectStatusProvider.updateWorkspace(cargoWorkspace);
	projectOutlineProvider.updateWorkspace(cargoWorkspace);
	await pinnedCommands.getTreeDataProvider().initialize();

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

	vscode.window.createTreeView('cargoToolsPinnedCommands', {
		treeDataProvider: pinnedCommands.getTreeDataProvider(),
		showCollapseAll: false,
		canSelectMany: false
	});

	// Subscribe to workspace changes to update providers
	if (cargoWorkspace) {
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
	}

	// Register task provider
	const taskProvider = new CargoTaskProvider(cargoWorkspace);
	context.subscriptions.push(
		vscode.tasks.registerTaskProvider(CargoTaskProvider.CargoType, taskProvider)
	);

	return {
		extensionManager,
		workspace: cargoWorkspace,
		projectStatusProvider,
		projectOutlineProvider,
		pinnedCommands
	};
}

/**
 * Helper function to set cargo context for when clauses
 */
async function setCargoContext(hasCargo: boolean): Promise<void> {
	await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasCargo', hasCargo);
	console.log(`Context set: cargoTools:workspaceHasCargo = ${hasCargo}`);
}

async function executeCargoCommand(command: string, workspace: CargoWorkspace): Promise<void> {
	const terminal = vscode.window.createTerminal({
		name: `Cargo ${command}`,
		cwd: workspace.workspaceRoot
	});

	const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');
	const args = workspace.getCargoArgs(command);
	const commandLine = `${cargoPath} ${args.join(' ')}`;

	terminal.sendText(commandLine);
	terminal.show();
}

function getTargetExecutablePath(workspace: CargoWorkspace): string {
	const target = workspace.currentTarget;
	if (!target) {
		throw new Error('No target selected');
	}

	const profile = workspace.currentProfile === CargoProfile.release ? 'release' : 'debug';
	return path.join(workspace.workspaceRoot, 'target', profile, target.name);
}

function hasCargoWorkspace(): boolean {
	const workspaceFolders = vscode.workspace.workspaceFolders;
	if (!workspaceFolders) {
		return false;
	}

	return workspaceFolders.some(folder => {
		const cargoToml = path.join(folder.uri.fsPath, 'Cargo.toml');
		return fs.existsSync(cargoToml);
	});
}

function getCargoWorkspaceRoot(): string | undefined {
	const workspaceFolders = vscode.workspace.workspaceFolders;
	if (!workspaceFolders) {
		return undefined;
	}

	const cargoFolder = workspaceFolders.find(folder => {
		const cargoToml = path.join(folder.uri.fsPath, 'Cargo.toml');
		return fs.existsSync(cargoToml);
	});

	return cargoFolder?.uri.fsPath;
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

		// Clear workspace reference
		cargoWorkspace = undefined;

		console.log('Cargo Tools extension deactivated successfully');
	} catch (error) {
		console.error('Error during Cargo Tools extension deactivation:', error);
	}
}
