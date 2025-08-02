import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { CargoWorkspace } from './cargoWorkspace';
import { CargoProfile } from './cargoProfile';
import { ProfilesTreeProvider } from './profilesTreeProvider';
import { TargetsTreeProvider } from './targetsTreeProvider';
import { WorkspaceTreeProvider } from './workspaceTreeProvider';
import { StatusBarProvider } from './statusBarProvider';
import { CargoTaskProvider } from './cargoTaskProvider';

let cargoWorkspace: CargoWorkspace | undefined;
let statusBarProvider: StatusBarProvider | undefined;

export async function activate(context: vscode.ExtensionContext) {
	try {
		console.log('Cargo Tools extension activation started...');

		// Check if workspace has Cargo.toml
		if (!hasCargoWorkspace()) {
			console.log('No Cargo workspace found - extension will not activate');
			return;
		}

		console.log('Cargo workspace detected!');

		// Set context for when clauses
		await vscode.commands.executeCommand('setContext', 'cargoTools:workspaceHasCargo', true);
		console.log('Context set: cargoTools:workspaceHasCargo = true');

		// Initialize cargo workspace
		const workspaceRoot = getCargoWorkspaceRoot();
		if (!workspaceRoot) {
			console.log('Could not determine workspace root');
			return;
		}

		console.log('Workspace root:', workspaceRoot);

		cargoWorkspace = new CargoWorkspace(workspaceRoot);
		await cargoWorkspace.initialize();

		// Create tree providers
		const profilesProvider = new ProfilesTreeProvider(cargoWorkspace);
		const targetsProvider = new TargetsTreeProvider(cargoWorkspace);
		const workspaceProvider = new WorkspaceTreeProvider(cargoWorkspace);

		// Register tree views
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

		// Create status bar provider
		statusBarProvider = new StatusBarProvider(cargoWorkspace);

		// Register task provider
		const taskProvider = new CargoTaskProvider(cargoWorkspace);
		context.subscriptions.push(
			vscode.tasks.registerTaskProvider(CargoTaskProvider.CargoType, taskProvider)
		);

		// Register commands
		registerCommands(context, cargoWorkspace, profilesProvider, targetsProvider, workspaceProvider);

		console.log('Cargo Tools extension fully initialized!');
	} catch (error) {
		console.error('Failed to activate Cargo Tools extension:', error);
		vscode.window.showErrorMessage(`Cargo Tools extension failed to activate: ${error}`);
	}
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

	// Select target command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.selectTarget', async (target?: any) => {
			if (!target) {
				const targets = workspace.targets;
				if (targets.length === 0) {
					vscode.window.showWarningMessage('No targets found in workspace');
					return;
				}

				const items = targets.map(t => ({
					label: t.name,
					description: t.kind.join(', '),
					detail: t.srcPath,
					target: t
				}));

				const selected = await vscode.window.showQuickPick(items, {
					placeHolder: 'Select build target'
				});

				if (selected) {
					target = selected.target;
				}
			}

			if (target) {
				workspace.setTarget(target);
				vscode.window.showInformationMessage(`Build target set to: ${target.name}`);
			}
		})
	);

	// Refresh command
	context.subscriptions.push(
		vscode.commands.registerCommand('cargo-tools.refresh', async () => {
			await workspace.refresh();
			profilesProvider.refresh();
			targetsProvider.refresh();
			workspaceProvider.refresh();
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
				const args = ['run', '--example', selected.target.name];
				
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

export function deactivate() {
	if (statusBarProvider) {
		statusBarProvider.dispose();
	}
}
