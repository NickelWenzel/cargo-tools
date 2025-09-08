import * as vscode from 'vscode';
import { ProjectStatusTreeProvider } from './projectStatusTreeProvider';
import { ProjectOutlineTreeProvider } from './projectOutlineTreeProvider';
import { MakefileTreeProvider } from './makefileTreeProvider';
import { PinnedMakefileTasksTreeProvider } from './pinnedMakefileTasksTreeProvider';
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

	// Set the Makefile context based on whether Makefile.toml exists
	await setMakefileContext(cargoWorkspace.hasMakefileToml);

	// New tree providers for the modern UI
	const projectStatusProvider = new ProjectStatusTreeProvider();
	const projectOutlineProvider = new ProjectOutlineTreeProvider();
	const makefileProvider = new MakefileTreeProvider();
	const pinnedMakefileTasksProvider = new PinnedMakefileTasksTreeProvider();

	// Update providers with workspace
	projectStatusProvider.updateWorkspace(cargoWorkspace);
	projectOutlineProvider.updateWorkspace(cargoWorkspace);
	makefileProvider.updateWorkspace(cargoWorkspace);
	pinnedMakefileTasksProvider.updateWorkspace(cargoWorkspace);

	// Register tree providers with extension manager for command access
	extensionManager.registerTreeProviders(projectOutlineProvider, projectStatusProvider, makefileProvider, pinnedMakefileTasksProvider);

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

	vscode.window.createTreeView('cargoToolsMakefile', {
		treeDataProvider: makefileProvider,
		showCollapseAll: true,
		canSelectMany: false
	});

	vscode.window.createTreeView('cargoToolsPinnedMakefileTasks', {
		treeDataProvider: pinnedMakefileTasksProvider,
		showCollapseAll: false,
		canSelectMany: false
	});

	// Register makefile task runner command
	const runMakeTaskDisposable = vscode.commands.registerCommand('cargo-tools.makefile.runTask',
		async (taskNameOrNode: string | any) => {
			try {
				if (!cargoWorkspace || !cargoWorkspace.hasMakefileToml) {
					vscode.window.showErrorMessage('No Makefile.toml found in workspace');
					return;
				}

				// Handle both string parameter (old usage) and node parameter (context menu)
				let taskName: string;
				if (typeof taskNameOrNode === 'string') {
					taskName = taskNameOrNode;
				} else if (taskNameOrNode?.data?.task?.name) {
					taskName = taskNameOrNode.data.task.name;
				} else {
					vscode.window.showErrorMessage('Unable to determine task name');
					return;
				}

				// Show which task is being run
				vscode.window.showInformationMessage(`Running cargo make task: ${taskName}`);

				// Create a terminal and run the cargo make command
				const terminal = vscode.window.createTerminal({
					name: `cargo make ${taskName}`,
					cwd: cargoWorkspace.workspaceRoot
				});

				terminal.sendText(`cargo make ${taskName}`);
				terminal.show();

			} catch (error) {
				const message = error instanceof Error ? error.message : String(error);
				vscode.window.showErrorMessage(`Failed to run cargo make task: ${message}`);
			}
		}
	);

	context.subscriptions.push(runMakeTaskDisposable);

	// Register makefile filter commands
	const setTaskFilterDisposable = vscode.commands.registerCommand('cargo-tools.makefile.setTaskFilter',
		async () => {
			await makefileProvider.setTaskFilter();
		}
	);

	const editTaskFilterDisposable = vscode.commands.registerCommand('cargo-tools.makefile.editTaskFilter',
		async () => {
			await makefileProvider.editTaskFilter();
		}
	);

	const clearTaskFilterDisposable = vscode.commands.registerCommand('cargo-tools.makefile.clearTaskFilter',
		() => {
			makefileProvider.clearTaskFilter();
		}
	);

	context.subscriptions.push(setTaskFilterDisposable);
	context.subscriptions.push(editTaskFilterDisposable);
	context.subscriptions.push(clearTaskFilterDisposable);

	// Register makefile category filter commands
	const showCategoryFilterDisposable = vscode.commands.registerCommand('cargo-tools.makefile.showCategoryFilter',
		async () => {
			await makefileProvider.showCategoryFilter();
		}
	);

	const clearCategoryFilterDisposable = vscode.commands.registerCommand('cargo-tools.makefile.clearCategoryFilter',
		() => {
			makefileProvider.clearCategoryFilter();
		}
	);

	context.subscriptions.push(showCategoryFilterDisposable);
	context.subscriptions.push(clearCategoryFilterDisposable);

	// Register pinned makefile tasks commands
	const addPinnedTaskDisposable = vscode.commands.registerCommand('cargo-tools.pinnedMakefileTasks.add',
		async () => {
			await pinnedMakefileTasksProvider.showAddTaskQuickPick();
		}
	);

	const removePinnedTaskDisposable = vscode.commands.registerCommand('cargo-tools.pinnedMakefileTasks.remove',
		async (node: any) => {
			if (node?.taskName) {
				await pinnedMakefileTasksProvider.removePinnedTask(node.taskName);
			}
		}
	);

	const executePinnedTaskDisposable = vscode.commands.registerCommand('cargo-tools.pinnedMakefileTasks.execute',
		async (node: any) => {
			try {
				if (!cargoWorkspace || !cargoWorkspace.hasMakefileToml) {
					vscode.window.showErrorMessage('No Makefile.toml found in workspace');
					return;
				}

				const taskName = node?.taskName;
				if (!taskName) {
					vscode.window.showErrorMessage('Unable to determine task name');
					return;
				}

				// Show which task is being run
				vscode.window.showInformationMessage(`Running cargo make task: ${taskName}`);

				// Create a terminal and run the cargo make command
				const terminal = vscode.window.createTerminal({
					name: `cargo make ${taskName}`,
					cwd: cargoWorkspace.workspaceRoot
				});

				terminal.sendText(`cargo make ${taskName}`);
				terminal.show();

			} catch (error) {
				const message = error instanceof Error ? error.message : String(error);
				vscode.window.showErrorMessage(`Failed to run cargo make task: ${message}`);
			}
		}
	);

	const pinTaskDisposable = vscode.commands.registerCommand('cargo-tools.makefile.pinTask',
		async (node: any) => {
			const taskName = node?.data?.task?.name;
			if (taskName) {
				await pinnedMakefileTasksProvider.addPinnedTask(taskName);
			}
		}
	);

	context.subscriptions.push(addPinnedTaskDisposable);
	context.subscriptions.push(removePinnedTaskDisposable);
	context.subscriptions.push(executePinnedTaskDisposable);
	context.subscriptions.push(pinTaskDisposable);

	// Subscribe to workspace changes to update providers
	cargoWorkspace.onDidChangeTargets(() => {
		projectStatusProvider.refresh();
		projectOutlineProvider.refresh();
		makefileProvider.refresh();
		pinnedMakefileTasksProvider.refresh();
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
 * Helper function to set makefile context for when clauses
 */
async function setMakefileContext(hasMakefile: boolean): Promise<void> {
	await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasMakefile', hasMakefile);
	console.log(`Context set: cargoTools:workspaceHasMakefile = ${hasMakefile}`);
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
