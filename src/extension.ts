import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';
import { ProfilesTreeProvider } from './profilesTreeProvider';
import { TargetsTreeProvider } from './targetsTreeProvider';
import { WorkspaceTreeProvider } from './workspaceTreeProvider';
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

	// Create tree providers (both old and new)
	const profilesProvider = new ProfilesTreeProvider(cargoWorkspace);
	const targetsProvider = new TargetsTreeProvider(cargoWorkspace);
	const workspaceProvider = new WorkspaceTreeProvider(cargoWorkspace);

	// New tree providers for the modern UI
	const projectStatusProvider = new ProjectStatusTreeProvider();
	const projectOutlineProvider = new ProjectOutlineTreeProvider();
	const pinnedCommands = new PinnedCommands(context);

	// Update providers with workspace
	projectStatusProvider.updateWorkspace(cargoWorkspace);
	projectOutlineProvider.updateWorkspace(cargoWorkspace);
	await pinnedCommands.getTreeDataProvider().initialize();

	// Register tree views (legacy views for backwards compatibility)
	vscode.window.createTreeView('cargoToolsProfiles', {
		treeDataProvider: profilesProvider,
		showCollapseAll: false
	});

	vscode.window.createTreeView('cargoToolsTargets', {
		treeDataProvider: targetsProvider,
		showCollapseAll: false
	});

	vscode.window.createTreeView('cargoToolsWorkspace', {
		treeDataProvider: workspaceProvider,
		showCollapseAll: true
	});

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

	// Register legacy commands - these will be gradually migrated to the extension manager
	// NOTE: Commands are now registered in the Extension Manager to prevent duplicates
	// registerCommands(context, cargoWorkspace, profilesProvider, targetsProvider, workspaceProvider);

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

function registerCommands(
	context: vscode.ExtensionContext,
	workspace: CargoWorkspace,
	profilesProvider: ProfilesTreeProvider,
	targetsProvider: TargetsTreeProvider,
	workspaceProvider: WorkspaceTreeProvider
) {
	// Build command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.build', async () => {
			await executeCargoCommand('build', workspace);
		})
	);

	// Run command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.run', async () => {
			if (!workspace.currentTarget?.isExecutable) {
				vscode.window.showWarningMessage('No executable target selected. Please select a binary target first.');
				return;
			}
			await executeCargoCommand('run', workspace);
		})
	);

	// Test command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.test', async () => {
			await executeCargoCommand('test', workspace);
		})
	);

	// Debug command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.debug', async () => {
			if (!workspace.currentTarget?.isExecutable) {
				vscode.window.showWarningMessage('No executable target selected. Please select a binary target first.');
				return;
			}

			// Build first, then start debugging
			try {
				await executeCargoCommand('build', workspace);

				// Start debugging session
				const debugConfig = {
					name: 'Debug Rust',
					type: 'cppdbg',
					request: 'launch',
					program: getTargetExecutablePath(workspace),
					cwd: workspace.workspaceRoot,
					args: [],
					stopAtEntry: false,
					environment: [],
					console: 'integratedTerminal'
				};

				await vscode.debug.startDebugging(undefined, debugConfig);
			} catch (error) {
				vscode.window.showErrorMessage(`Debug failed: ${error}`);
			}
		})
	);

	// Clean command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.clean', async () => {
			await executeCargoCommand('clean', workspace);
		})
	);

	// Select profile command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.selectProfile', async (profile?: CargoProfile) => {
			if (!profile) {
				const profiles = CargoProfile.getAllProfiles();
				const items = profiles.map(p => ({
					label: CargoProfile.getDisplayName(p),
					description: CargoProfile.getDescription(p),
					profile: p
				}));

				const selected = await vscode.window.showQuickPick(items, {
					placeHolder: 'Select build profile'
				});

				if (selected) {
					profile = selected.profile;
				}
			}

			if (profile) {
				workspace.setProfile(profile);
				vscode.window.showInformationMessage(`Build profile set to: ${CargoProfile.getDisplayName(profile)}`);
			}
		})
	);

	// Refresh command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.refresh', async () => {
			await workspace.refresh();
			// Tree providers will automatically refresh via event subscriptions
			vscode.window.showInformationMessage('Cargo workspace refreshed');
		})
	);

	// Edit configuration command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.editConfiguration', async () => {
			await vscode.commands.executeCommand('workbench.action.openSettings', 'cargoTools');
		})
	);

	// Run example command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.runExample', async () => {
			const examples = workspace.targets.filter(t => t.isExample);
			if (examples.length === 0) {
				vscode.window.showWarningMessage('No examples found in workspace');
				return;
			}

			const items = examples.map(example => ({
				label: example.name,
				description: 'example',
				detail: example.srcPath,
				target: example
			}));

			const selected = await vscode.window.showQuickPick(items, {
				placeHolder: 'Select example to run'
			});

			if (selected) {
				const terminal = vscode.window.createTerminal({
					name: `Example: ${selected.target.name}`,
					cwd: workspace.workspaceRoot
				});

				const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');
				const args = ['run'];

				// Add package argument if we have package info and it's a workspace
				if (selected.target.packageName && isWorkspace(workspace)) {
					args.push('--package', selected.target.packageName);
				}

				args.push('--example', selected.target.name);

				if (workspace.currentProfile.toString() === 'release') {
					args.push('--release');
				}

				terminal.sendText(`${cargoPath} ${args.join(' ')}`);
				terminal.show();
			}
		})
	);

	// Run test command (specific test)
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.runTest', async () => {
			const tests = workspace.targets.filter(t => t.isTest);

			const items = [
				{ label: 'All tests', description: 'Run all tests', target: null },
				...tests.map(test => ({
					label: test.name,
					description: 'integration test',
					detail: test.srcPath,
					target: test
				}))
			];

			const selected = await vscode.window.showQuickPick(items, {
				placeHolder: 'Select test to run'
			});

			if (selected) {
				const terminal = vscode.window.createTerminal({
					name: selected.target ? `Test: ${selected.target.name}` : 'All Tests',
					cwd: workspace.workspaceRoot
				});

				const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');
				const args = ['test'];

				if (selected.target) {
					// Add package argument if we have package info and it's a workspace
					if (selected.target.packageName && isWorkspace(workspace)) {
						args.push('--package', selected.target.packageName);
					}
					args.push('--test', selected.target.name);
				}

				if (workspace.currentProfile.toString() === 'release') {
					args.push('--release');
				}

				terminal.sendText(`${cargoPath} ${args.join(' ')}`);
				terminal.show();
			}
		})
	);

	// Run benchmark command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.runBench', async () => {
			const benches = workspace.targets.filter(t => t.isBench);
			if (benches.length === 0) {
				vscode.window.showWarningMessage('No benchmarks found in workspace');
				return;
			}

			const items = [
				{ label: 'All benchmarks', description: 'Run all benchmarks', target: null },
				...benches.map(bench => ({
					label: bench.name,
					description: 'benchmark',
					detail: bench.srcPath,
					target: bench
				}))
			];

			const selected = await vscode.window.showQuickPick(items, {
				placeHolder: 'Select benchmark to run'
			});

			if (selected) {
				const terminal = vscode.window.createTerminal({
					name: selected.target ? `Bench: ${selected.target.name}` : 'All Benchmarks',
					cwd: workspace.workspaceRoot
				});

				const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');
				const args = ['bench'];

				if (selected.target) {
					// Add package argument if we have package info and it's a workspace
					if (selected.target.packageName && isWorkspace(workspace)) {
						args.push('--package', selected.target.packageName);
					}
					args.push('--bench', selected.target.name);
				}

				terminal.sendText(`${cargoPath} ${args.join(' ')}`);
				terminal.show();
			}
		})
	);
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
