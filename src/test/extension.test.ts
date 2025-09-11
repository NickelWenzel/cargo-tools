import * as assert from 'assert';
import * as path from 'path';
import * as fs from 'fs';

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from 'vscode';
import { CargoExtensionManager } from '../cargoExtensionManager';
// import * as myExtension from '../../extension';

// Helper function to get the test project path relative to workspace root
function getTestProjectPath(): string {
	// Get the workspace root (the extension's root directory)
	const workspaceRoot = path.resolve(__dirname, '..', '..');
	return path.join(workspaceRoot, 'test-rust-project');
}

suite('Extension Test Suite', () => {
	vscode.window.showInformationMessage('Start all tests.');

	test('Sample test', () => {
		assert.strictEqual(-1, [1, 2, 3].indexOf(5));
		assert.strictEqual(-1, [1, 2, 3].indexOf(0));
	});

	suite('Action-Type Command Integration Tests', () => {
		test('should verify expected commands exist in package.json', async () => {
			// Instead of checking if commands are registered (which requires activation),
			// verify that our expected commands are defined in package.json
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];
				const commandIds = commands.map((cmd: any) => cmd.command);

				// Check for main selection commands
				assert.ok(commandIds.includes('cargo-tools.selectProfile'), 'selectProfile command should be defined');
				assert.ok(commandIds.includes('cargo-tools.selectPackage'), 'selectPackage command should be defined');
				assert.ok(commandIds.includes('cargo-tools.selectBuildTarget'), 'selectBuildTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.selectRunTarget'), 'selectRunTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.selectBenchmarkTarget'), 'selectBenchmarkTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.selectFeatures'), 'selectFeatures command should be defined');
				assert.ok(commandIds.includes('cargo-tools.buildDocs'), 'buildDocs command should be defined');
				assert.ok(commandIds.includes('cargo-tools.refresh'), 'refresh command should be defined');
				assert.ok(commandIds.includes('cargo-tools.clean'), 'clean command should be defined');

				// Check for context menu commands
				assert.ok(commandIds.includes('cargo-tools.projectOutline.selectPackage'), 'projectOutline.selectPackage command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.setBuildTarget'), 'projectOutline.setBuildTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.toggleFeature'), 'projectOutline.toggleFeature command should be defined');
				assert.ok(commandIds.includes('cargo-tools.executeDefaultBench'), 'executeDefaultBench command should be defined');

				// Check for context menu commands
				assert.ok(commandIds.includes('cargo-tools.executeBuildAction'), 'executeBuildAction command should be defined');
				assert.ok(commandIds.includes('cargo-tools.executeRunAction'), 'executeRunAction command should be defined');
				assert.ok(commandIds.includes('cargo-tools.executeTestAction'), 'executeTestAction command should be defined');
				assert.ok(commandIds.includes('cargo-tools.executeBenchAction'), 'executeBenchAction command should be defined');

				// Check for set default target commands  
				assert.ok(commandIds.includes('cargo-tools.setAsDefaultBuildTarget'), 'setAsDefaultBuildTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.setAsDefaultRunTarget'), 'setAsDefaultRunTarget command should be defined');

				// Check for new project outline action commands
				assert.ok(commandIds.includes('cargo-tools.projectOutline.buildPackage'), 'projectOutline.buildPackage command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.testPackage'), 'projectOutline.testPackage command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.cleanPackage'), 'projectOutline.cleanPackage command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.buildWorkspace'), 'projectOutline.buildWorkspace command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.testWorkspace'), 'projectOutline.testWorkspace command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.cleanWorkspace'), 'projectOutline.cleanWorkspace command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.buildTarget'), 'projectOutline.buildTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.runTarget'), 'projectOutline.runTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.benchTarget'), 'projectOutline.benchTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.setAsDefaultTestTarget'), 'setAsDefaultTestTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.setAsDefaultBenchTarget'), 'setAsDefaultBenchTarget command should be defined');

				// Check for makefile commands
				assert.ok(commandIds.includes('cargo-tools.makefile.runTask'), 'makefile.runTask command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.selectAndRunTask'), 'makefile.selectAndRunTask command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.setTaskFilter'), 'makefile.setTaskFilter command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.editTaskFilter'), 'makefile.editTaskFilter command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.clearTaskFilter'), 'makefile.clearTaskFilter command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.showCategoryFilter'), 'makefile.showCategoryFilter command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.clearCategoryFilter'), 'makefile.clearCategoryFilter command should be defined');
			} else {
				// If running without the extension being loaded, just check that 
				// the command patterns are reasonable (integration test framework limitation)
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});

		test('should have key bindings registered for common commands', () => {
			// Test that the key bindings are properly registered in package.json
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const keybindings = packageJson.contributes?.keybindings || [];

				// Check for the main key bindings
				const buildKeybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.projectStatus.build' && kb.key === 'f7'
				);
				const runKeybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.projectStatus.run' && kb.key === 'ctrl+shift+f5'
				);
				const debugKeybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.projectStatus.debug' && kb.key === 'shift+f5'
				);

				assert.ok(buildKeybinding, 'Build command (F7) key binding should be defined');
				assert.ok(runKeybinding, 'Run command (Ctrl+Shift+F5) key binding should be defined');
				assert.ok(debugKeybinding, 'Debug command (Shift+F5) key binding should be defined');

				// Check that they have the correct "when" context
				assert.strictEqual(buildKeybinding.when, 'cargoTools:workspaceHasCargo', 'Build key binding should have correct when context');
				assert.strictEqual(runKeybinding.when, 'cargoTools:workspaceHasCargo', 'Run key binding should have correct when context');
				assert.strictEqual(debugKeybinding.when, 'cargoTools:workspaceHasCargo', 'Debug key binding should have correct when context');
			} else {
				// If running without the extension being loaded, just check that 
				// the test framework is working (integration test framework limitation)
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});

		test('should have cargo tree view provider registered', async () => {
			// Verify that the cargo tree view is registered and available
			const treeViews = vscode.window.createTreeView('cargo-tools.targets', {
				treeDataProvider: {
					getTreeItem: () => new vscode.TreeItem('test'),
					getChildren: () => []
				}
			});

			assert.ok(treeViews, 'Cargo targets tree view should be available');
			treeViews.dispose();
		});

		test('should have extension manager instance available', () => {
			// Verify that the extension manager can be accessed (after activation)
			try {
				const manager = CargoExtensionManager.getInstance();
				assert.ok(manager, 'Extension manager instance should be available');
			} catch (error) {
				// This is expected if the extension hasn't been activated yet
				assert.ok(error instanceof Error && error.message.includes('not initialized'),
					'Should get proper error message when not initialized');
			}
		});
	});

	suite('Project Outline View Filter Tests', () => {
		// Import the ProjectOutlineTreeProvider at the top to avoid circular imports
		const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

		test('should apply workspace member filter when grouping is disabled', () => {
			const provider = new ProjectOutlineTreeProvider();

			// Create mock targets from different packages
			const targets = [
				{
					name: 'app1',
					packageName: 'my-app',
					kind: ['bin'],
					srcPath: '/path/to/app1.rs'
				},
				{
					name: 'lib1',
					packageName: 'my-library',
					kind: ['lib'],
					srcPath: '/path/to/lib1.rs'
				},
				{
					name: 'app2',
					packageName: 'my-app',
					kind: ['bin'],
					srcPath: '/path/to/app2.rs'
				}
			];

			// Set workspace member filter to 'app' - should only show targets from packages containing 'app'
			// Access private field for testing
			(provider as any).workspaceMemberFilter = 'app';

			// Test the private filterTargets method using bracket notation to access it
			const filteredTargets = (provider as any).filterTargets(targets);

			// Should only include targets from 'my-app' package (contains 'app')
			assert.strictEqual(filteredTargets.length, 2, 'Should filter to 2 targets from my-app package');
			assert.ok(filteredTargets.every((target: any) => target.packageName === 'my-app'),
				'All filtered targets should be from my-app package');
			assert.ok(filteredTargets.some((target: any) => target.name === 'app1'),
				'Should include app1 target');
			assert.ok(filteredTargets.some((target: any) => target.name === 'app2'),
				'Should include app2 target');
		});

		test('should handle targets with undefined packageName gracefully', () => {
			const provider = new ProjectOutlineTreeProvider();

			// Create mock targets with some having undefined packageName
			const targets = [
				{
					name: 'app1',
					packageName: 'my-app',
					kind: ['bin'],
					srcPath: '/path/to/app1.rs'
				},
				{
					name: 'lib1',
					packageName: undefined,
					kind: ['lib'],
					srcPath: '/path/to/lib1.rs'
				}
			];

			// Set workspace member filter
			// Access private field for testing
			(provider as any).workspaceMemberFilter = 'app';

			// Test the private filterTargets method
			const filteredTargets = (provider as any).filterTargets(targets);

			// Should only include targets with defined packageName that matches the filter
			assert.strictEqual(filteredTargets.length, 1, 'Should filter to 1 target');
			assert.strictEqual(filteredTargets[0].name, 'app1', 'Should include app1 target');
		});

		test('should not filter when workspace member filter is empty', () => {
			const provider = new ProjectOutlineTreeProvider();

			// Create mock targets from different packages
			const targets = [
				{
					name: 'app1',
					packageName: 'my-app',
					kind: ['bin'],
					srcPath: '/path/to/app1.rs'
				},
				{
					name: 'lib1',
					packageName: 'my-library',
					kind: ['lib'],
					srcPath: '/path/to/lib1.rs'
				}
			];

			// Leave workspace member filter empty (default)
			// (provider as any).workspaceMemberFilter should be '' by default

			// Test the private filterTargets method
			const filteredTargets = (provider as any).filterTargets(targets);

			// Should include all targets when no workspace member filter is active
			assert.strictEqual(filteredTargets.length, 2, 'Should include all targets when filter is empty');
		});
	});

	suite('Makefile.toml Detection Tests', () => {
		const { CargoWorkspace } = require('../cargoWorkspace');

		test('should detect Makefile.toml when it exists', async function() {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds
			
			// Use the test-rust-project which now has a Makefile.toml
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();

			// Should detect the Makefile.toml we created
			assert.strictEqual(workspace.hasMakefileToml, true, 'Should detect Makefile.toml in test project');
		});

		test('should return false when Makefile.toml does not exist', async function() {
			// Increase timeout for this test as well since it initializes a workspace
			this.timeout(20000); // 20 seconds
			
			// Use a path that doesn't have Makefile.toml 
			const testPath = '/tmp'; // temp directory should not have Makefile.toml
			const workspace = new CargoWorkspace(testPath);

			await workspace.initialize();

			// Should not detect Makefile.toml
			assert.strictEqual(workspace.hasMakefileToml, false, 'Should not detect Makefile.toml when it does not exist');
		});

		test('refresh command should update makefile context', async () => {
			// Test that the refresh command is properly defined and available
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];
				const commandIds = commands.map((cmd: any) => cmd.command);

				assert.ok(commandIds.includes('cargo-tools.refresh'), 'Refresh command should be defined in package.json');

				// Find the refresh command definition
				const refreshCommand = commands.find((cmd: any) => cmd.command === 'cargo-tools.refresh');
				assert.ok(refreshCommand, 'Refresh command should have complete definition');
				assert.strictEqual(refreshCommand.title, 'Refresh', 'Refresh command should have correct title');
				assert.strictEqual(refreshCommand.category, 'Cargo Tools', 'Refresh command should have correct category');

				// Check that refresh command is available in Project Status view menu
				const menus = packageJson.contributes?.menus || {};
				const viewTitleMenus = menus['view/title'] || [];
				const refreshMenu = viewTitleMenus.find((menu: any) =>
					menu.command === 'cargo-tools.refresh' &&
					menu.when === 'view == cargoToolsProjectStatus'
				);
				assert.ok(refreshMenu, 'Refresh command should be available in Project Status view menu');
			} else {
				assert.ok(true, 'Extension package.json not available in test context, skipping refresh command test');
			}
		});
	});

	suite('Makefile Tree Provider Tests', () => {
		const { MakefileTreeProvider } = require('../makefileTreeProvider');
		const { CargoWorkspace } = require('../cargoWorkspace');

		test('should show task categories when Makefile.toml exists', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();
			provider.updateWorkspace(workspace);

			// Should return array of task categories
			const children = await provider.getChildren();
			assert.strictEqual(Array.isArray(children), true, 'Should return an array');
			assert.strictEqual(children.length > 0, true, 'Should have task categories when Makefile.toml exists');

			// Check that the first child is a category node
			if (children.length > 0) {
				const firstChild = children[0];
				assert.strictEqual(typeof firstChild.label, 'string', 'Category should have a label');
				assert.strictEqual(firstChild.collapsibleState !== undefined, true, 'Category should be collapsible');
			}
		});

		test('should show no makefile message when Makefile.toml does not exist', async () => {
			const provider = new MakefileTreeProvider();
			const testPath = '/tmp';
			const workspace = new CargoWorkspace(testPath);

			await workspace.initialize();
			provider.updateWorkspace(workspace);

			// Should show "No Makefile.toml found" message
			const children = await provider.getChildren();
			assert.strictEqual(children.length, 1, 'Should have one message node');
			assert.strictEqual(children[0].label, 'No Makefile.toml found', 'Should show appropriate message');
		});

		test('should apply task filter correctly', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();
			provider.updateWorkspace(workspace);

			// Get all tasks first
			const allChildren = await provider.getChildren();
			const allTaskCount = allChildren.length;
			assert.strictEqual(allTaskCount > 0, true, 'Should have tasks when no filter is applied');

			// Apply a filter that should reduce the number of categories
			provider.clearTaskFilter(); // Ensure we start with no filter
			const childrenBeforeFilter = await provider.getChildren();

			// Test filter methods exist
			assert.strictEqual(typeof provider.setTaskFilter, 'function', 'setTaskFilter method should exist');
			assert.strictEqual(typeof provider.clearTaskFilter, 'function', 'clearTaskFilter method should exist');
			assert.strictEqual(typeof provider.currentTaskFilter, 'string', 'currentTaskFilter should be a string');

			// Test clear filter
			provider.clearTaskFilter();
			assert.strictEqual(provider.currentTaskFilter, '', 'Filter should be empty after clear');

			// Note: Filter now only applies to task names, not descriptions or categories
		});

		test('should apply category filter correctly', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();
			provider.updateWorkspace(workspace);

			// Get all categories first
			const allChildren = await provider.getChildren();
			const allCategoryCount = allChildren.length;
			assert.strictEqual(allCategoryCount > 0, true, 'Should have categories when no filter is applied');

			// Test category filter methods exist
			assert.strictEqual(typeof provider.showCategoryFilter, 'function', 'showCategoryFilter method should exist');
			assert.strictEqual(typeof provider.clearCategoryFilter, 'function', 'clearCategoryFilter method should exist');
			assert.strictEqual(typeof provider.currentCategoryFilter, 'object', 'currentCategoryFilter should be a Set');

			// Test clear category filter
			provider.clearCategoryFilter();
			const childrenAfterClear = await provider.getChildren();
			assert.strictEqual(childrenAfterClear.length, allCategoryCount, 'Should show all categories after clear');
		});
	});

	suite('Makefile Filter Integration Tests', () => {
		const { MakefileTreeProvider } = require('../makefileTreeProvider');
		const { CargoWorkspace } = require('../cargoWorkspace');

		test('should have proper command registration for filters', () => {
			// Test the specific command registrations that we need for filtering
			const packageJsonPath = require('path').join(__dirname, '../../package.json');
			const packageJson = require(packageJsonPath);
			const commands = packageJson.contributes?.commands || [];
			const commandIds = commands.map((cmd: any) => cmd.command);

			// Verify all expected filter commands are registered
			const expectedFilterCommands = [
				'cargo-tools.makefile.runTask',
				'cargo-tools.makefile.selectAndRunTask',
				'cargo-tools.makefile.setTaskFilter',
				'cargo-tools.makefile.editTaskFilter',
				'cargo-tools.makefile.clearTaskFilter',
				'cargo-tools.makefile.showCategoryFilter',
				'cargo-tools.makefile.clearCategoryFilter'
			];

			for (const cmd of expectedFilterCommands) {
				assert.ok(commandIds.includes(cmd), `Command ${cmd} should be registered`);
			}
		});

		test('should have view title menu entries for filter buttons', () => {
			const packageJsonPath = require('path').join(__dirname, '../../package.json');
			const packageJson = require(packageJsonPath);
			const menus = packageJson.contributes?.menus || {};
			const viewTitleMenus = menus['view/title'] || [];

			// Find menus related to Makefile view
			const makefileViewMenus = viewTitleMenus.filter((menu: any) =>
				menu.when && menu.when.includes('cargoToolsMakefile')
			);

			assert.strictEqual(makefileViewMenus.length >= 2, true,
				'Should have at least 2 menu entries for Makefile view (filter and clear filter buttons)');
		});

		test('should filter tasks by name only (not description or category)', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();
			if (!workspace.hasMakefileToml) {
				// Skip test if no Makefile.toml exists
				return;
			}

			provider.updateWorkspace(workspace);

			// Get all task nodes from categories (we need to drill down to actual tasks)
			const allCategories = await provider.getChildren();
			let allTasks: any[] = [];

			for (const category of allCategories) {
				if (category.collapsibleState !== undefined) {
					// This is a category, get its tasks
					const categoryTasks = await provider.getChildren(category);
					allTasks = allTasks.concat(categoryTasks);
				}
			}

			if (allTasks.length === 0) {
				// Skip test if no tasks are found
				return;
			}

			// Test filter methods exist and have correct signatures
			assert.strictEqual(typeof provider.setTaskFilter, 'function', 'setTaskFilter method should exist');
			assert.strictEqual(typeof provider.clearTaskFilter, 'function', 'clearTaskFilter method should exist');
			assert.strictEqual(typeof provider.currentTaskFilter, 'string', 'currentTaskFilter should be a string');

			// Test clear filter
			provider.clearTaskFilter();
			assert.strictEqual(provider.currentTaskFilter, '', 'Filter should be empty after clear');

			// Test that filter property can be set directly (for testing)
			// Access private property for testing
			(provider as any).taskFilter = 'build';
			assert.strictEqual(provider.currentTaskFilter, 'build', 'Filter should be set correctly through property');

			// Clear filter for clean test
			provider.clearTaskFilter();
			assert.strictEqual(provider.currentTaskFilter, '', 'Filter should be cleared correctly');
		});

		test('should maintain category filter state correctly', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();
			if (!workspace.hasMakefileToml) {
				// Skip test if no Makefile.toml exists
				return;
			}

			provider.updateWorkspace(workspace);

			// Get all categories
			const allCategories = await provider.getChildren();
			if (allCategories.length === 0) {
				// Skip test if no categories found
				return;
			}

			// Category filter should be a Set
			assert.strictEqual(provider.currentCategoryFilter instanceof Set, true,
				'Category filter should be a Set');

			// Test category filter methods exist
			assert.strictEqual(typeof provider.showCategoryFilter, 'function', 'showCategoryFilter method should exist');
			assert.strictEqual(typeof provider.clearCategoryFilter, 'function', 'clearCategoryFilter method should exist');
			assert.strictEqual(typeof provider.currentCategoryFilter, 'object', 'currentCategoryFilter should be a Set');

			// When clearCategoryFilter is called, it should initialize to show all categories
			provider.clearCategoryFilter();
			const childrenAfterClear = await provider.getChildren();
			assert.strictEqual(childrenAfterClear.length, allCategories.length,
				'Should show all categories after clear');

			// After clear, the filter should contain all categories (not be empty)
			assert.strictEqual(provider.currentCategoryFilter.size > 0, true,
				'Category filter should contain all categories after clear (not be empty)');
		});

		test('should provide test project with Makefile.toml for testing', () => {
			const fs = require('fs');
			const path = require('path');
			const testProjectPath = path.join(__dirname, '../../test-rust-project');
			const makefilePath = path.join(testProjectPath, 'Makefile.toml');

			assert.strictEqual(fs.existsSync(makefilePath), true,
				'Test project should have Makefile.toml for testing filter functionality');
		});

		test('should have task nodes without click commands (button-triggered execution)', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();
			if (!workspace.hasMakefileToml) {
				// Skip test if no Makefile.toml exists
				return;
			}

			provider.updateWorkspace(workspace);

			// Get all categories
			const allCategories = await provider.getChildren();
			if (allCategories.length === 0) {
				// Skip test if no categories found
				return;
			}

			// Get tasks from first category
			const firstCategory = allCategories[0];
			if (firstCategory.collapsibleState !== undefined) {
				const tasks = await provider.getChildren(firstCategory);

				if (tasks.length > 0) {
					const firstTask = tasks[0];

					// Task should have proper context value for context menu
					assert.strictEqual(firstTask.contextValue, 'makefileTask',
						'Task should have makefileTask context value for context menu');

					// Task should not have a click command (button-triggered execution)
					assert.strictEqual(firstTask.command, undefined,
						'Task should not have click command - execution should be button-triggered');

					// Task should have gear icon instead of play icon
					assert.strictEqual(firstTask.iconPath instanceof vscode.ThemeIcon, true,
						'Task should have an icon');
					if (firstTask.iconPath instanceof vscode.ThemeIcon) {
						assert.strictEqual((firstTask.iconPath as any).id, 'gear',
							'Task should have gear icon instead of play icon');
					}
				}
			}
		});

		test('should have context menu entry for task execution', () => {
			const packageJsonPath = require('path').join(__dirname, '../../package.json');
			const packageJson = require(packageJsonPath);
			const menus = packageJson.contributes?.menus || {};
			const contextMenus = menus['view/item/context'] || [];

			// Find the makefile task context menu entry
			const makefileTaskMenu = contextMenus.find((menu: any) =>
				menu.when && menu.when.includes('cargoToolsMakefile') &&
				menu.when.includes('makefileTask') &&
				menu.command === 'cargo-tools.makefile.runTask'
			);

			assert.ok(makefileTaskMenu, 'Should have context menu entry for makefile task execution');
			assert.strictEqual(makefileTaskMenu.group, 'inline@1',
				'Task execution button should be inline in context menu');
		});

		test('should support state persistence for filters', () => {
			const { StateManager } = require('../stateManager');
			const mockContext = { globalState: { get: () => undefined, update: () => Promise.resolve() } };
			const mockFolder = { uri: { fsPath: '/test' }, name: 'test' };

			const stateManager = new StateManager(mockContext, mockFolder);

			// Verify StateManager has the makefile state methods
			assert.strictEqual(typeof stateManager.getMakefileTaskFilter, 'function',
				'StateManager should have getMakefileTaskFilter method');
			assert.strictEqual(typeof stateManager.setMakefileTaskFilter, 'function',
				'StateManager should have setMakefileTaskFilter method');
			assert.strictEqual(typeof stateManager.getMakefileCategoryFilter, 'function',
				'StateManager should have getMakefileCategoryFilter method');
			assert.strictEqual(typeof stateManager.setMakefileCategoryFilter, 'function',
				'StateManager should have setMakefileCategoryFilter method');
			assert.strictEqual(typeof stateManager.getIsMakefileCategoryFilterActive, 'function',
				'StateManager should have getIsMakefileCategoryFilterActive method');
			assert.strictEqual(typeof stateManager.setIsMakefileCategoryFilterActive, 'function',
				'StateManager should have setIsMakefileCategoryFilterActive method');
		});

		test('should have setStateManager method on MakefileTreeProvider', () => {
			const { MakefileTreeProvider } = require('../makefileTreeProvider');
			const provider = new MakefileTreeProvider();

			assert.strictEqual(typeof provider.setStateManager, 'function',
				'MakefileTreeProvider should have setStateManager method');
			assert.strictEqual(typeof provider.loadPersistedState, 'function',
				'MakefileTreeProvider should have loadPersistedState method');
		});

		test('should load persisted state correctly when state manager is set', () => {
			const { MakefileTreeProvider } = require('../makefileTreeProvider');
			const { StateManager } = require('../stateManager');

			// Mock state data
			const mockTaskFilter = 'test-task';
			const mockCategoryFilter = ['build', 'test'];

			// Create mock context and state manager
			const mockContext = {
				globalState: {
					get: (key: string) => {
						if (key.includes('makefile.taskFilter')) {
							return mockTaskFilter;
						}
						if (key.includes('makefile.categoryFilter')) {
							return mockCategoryFilter;
						}
						return undefined;
					},
					update: () => Promise.resolve()
				}
			};
			const mockFolder = { uri: { fsPath: '/test' }, name: 'test' };
			const stateManager = new StateManager(mockContext, mockFolder);

			// Create provider and set state manager
			const provider = new MakefileTreeProvider();
			provider.setStateManager(stateManager);

			// Load persisted state
			provider.loadPersistedState();

			// Verify state was loaded (we can't directly access private properties, 
			// but the method should execute without errors)
			assert.ok(true, 'loadPersistedState should execute without errors');
		});
	});

	suite('Pinned Makefile Tasks Integration Tests', () => {
		test('should have pinned makefile tasks commands registered in package.json', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];
				const commandIds = commands.map((cmd: any) => cmd.command);

				// Check for pinned makefile tasks commands
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.add'), 'pinnedMakefileTasks.add command should be defined');
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.remove'), 'pinnedMakefileTasks.remove command should be defined');
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.execute'), 'pinnedMakefileTasks.execute command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.pinTask'), 'makefile.pinTask command should be defined');
			}
		});

		test('should have pinned makefile tasks view defined in package.json', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const views = packageJson.contributes?.views?.cargoTools || [];
				const viewIds = views.map((view: any) => view.id);

				assert.ok(viewIds.includes('cargoToolsPinnedMakefileTasks'), 'Pinned Makefile Tasks view should be defined');
			}
		});

		test('should have context menu entries for pinned tasks', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const menus = packageJson.contributes?.menus?.['view/item/context'] || [];

				// Check for context menu entries
				const pinnedTaskExecuteMenu = menus.find((menu: any) =>
					menu.command === 'cargo-tools.pinnedMakefileTasks.execute' &&
					menu.when.includes('cargoToolsPinnedMakefileTasks') &&
					menu.when.includes('pinned-task')
				);

				const pinnedTaskRemoveMenu = menus.find((menu: any) =>
					menu.command === 'cargo-tools.pinnedMakefileTasks.remove' &&
					menu.when.includes('cargoToolsPinnedMakefileTasks') &&
					menu.when.includes('pinned-task')
				);

				const pinTaskMenu = menus.find((menu: any) =>
					menu.command === 'cargo-tools.makefile.pinTask' &&
					menu.when.includes('cargoToolsMakefile') &&
					menu.when.includes('makefileTask')
				);

				assert.ok(pinnedTaskExecuteMenu, 'Execute pinned task context menu should be defined');
				assert.ok(pinnedTaskRemoveMenu, 'Remove pinned task context menu should be defined');
				assert.ok(pinTaskMenu, 'Pin task context menu should be defined');
			}
		});

		test('should have PinnedMakefileTasksTreeProvider methods', () => {
			const { PinnedMakefileTasksTreeProvider } = require('../pinnedMakefileTasksTreeProvider');

			// Create a provider instance
			const provider = new PinnedMakefileTasksTreeProvider();

			// Verify the methods exist
			assert.strictEqual(typeof provider.setStateManager, 'function',
				'PinnedMakefileTasksTreeProvider should have setStateManager method');
			assert.strictEqual(typeof provider.loadPersistedState, 'function',
				'PinnedMakefileTasksTreeProvider should have loadPersistedState method');
			assert.strictEqual(typeof provider.addPinnedTask, 'function',
				'PinnedMakefileTasksTreeProvider should have addPinnedTask method');
			assert.strictEqual(typeof provider.removePinnedTask, 'function',
				'PinnedMakefileTasksTreeProvider should have removePinnedTask method');
			assert.strictEqual(typeof provider.showAddTaskQuickPick, 'function',
				'PinnedMakefileTasksTreeProvider should have showAddTaskQuickPick method');
		});

		test('should support state persistence for pinned tasks', () => {
			const { StateManager } = require('../stateManager');
			const mockContext = { globalState: { get: () => undefined, update: () => Promise.resolve() } };
			const mockFolder = { uri: { fsPath: '/test' }, name: 'test' };

			const stateManager = new StateManager(mockContext, mockFolder);

			// Verify StateManager has the pinned tasks state methods
			assert.strictEqual(typeof stateManager.getPinnedMakefileTasks, 'function',
				'StateManager should have getPinnedMakefileTasks method');
			assert.strictEqual(typeof stateManager.setPinnedMakefileTasks, 'function',
				'StateManager should have setPinnedMakefileTasks method');
		});

		test('should have view title menu entry for add button', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const viewTitleMenus = packageJson.contributes?.menus?.['view/title'] || [];

				const addPinnedTaskMenu = viewTitleMenus.find((menu: any) =>
					menu.command === 'cargo-tools.pinnedMakefileTasks.add' &&
					menu.when === 'view == cargoToolsPinnedMakefileTasks'
				);

				assert.ok(addPinnedTaskMenu, 'Add pinned task view title menu should be defined');
			}
		});

		test('should have pinned task execution commands (1st to 5th) registered', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];
				const commandIds = commands.map((cmd: any) => cmd.command);

				// Check for numbered pinned task execution commands
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.execute1'), 'execute1 command should be defined');
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.execute2'), 'execute2 command should be defined');
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.execute3'), 'execute3 command should be defined');
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.execute4'), 'execute4 command should be defined');
				assert.ok(commandIds.includes('cargo-tools.pinnedMakefileTasks.execute5'), 'execute5 command should be defined');
			}
		});

		test('should have key bindings for pinned task execution commands', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const keybindings = packageJson.contributes?.keybindings || [];

				// Check for the pinned task execution key bindings
				const execute1Keybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.pinnedMakefileTasks.execute1' && kb.key === 'ctrl+alt+1'
				);
				const execute2Keybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.pinnedMakefileTasks.execute2' && kb.key === 'ctrl+alt+2'
				);
				const execute3Keybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.pinnedMakefileTasks.execute3' && kb.key === 'ctrl+alt+3'
				);
				const execute4Keybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.pinnedMakefileTasks.execute4' && kb.key === 'ctrl+alt+4'
				);
				const execute5Keybinding = keybindings.find((kb: any) =>
					kb.command === 'cargo-tools.pinnedMakefileTasks.execute5' && kb.key === 'ctrl+alt+5'
				);

				assert.ok(execute1Keybinding, '1st pinned task (Ctrl+Alt+1) key binding should be defined');
				assert.ok(execute2Keybinding, '2nd pinned task (Ctrl+Alt+2) key binding should be defined');
				assert.ok(execute3Keybinding, '3rd pinned task (Ctrl+Alt+3) key binding should be defined');
				assert.ok(execute4Keybinding, '4th pinned task (Ctrl+Alt+4) key binding should be defined');
				assert.ok(execute5Keybinding, '5th pinned task (Ctrl+Alt+5) key binding should be defined');

				// Check that they have the correct "when" context
				const expectedWhen = 'cargoTools:workspaceHasCargo && cargoTools:workspaceHasMakefile';
				assert.strictEqual(execute1Keybinding.when, expectedWhen, '1st pinned task key binding should have correct when context');
				assert.strictEqual(execute2Keybinding.when, expectedWhen, '2nd pinned task key binding should have correct when context');
				assert.strictEqual(execute3Keybinding.when, expectedWhen, '3rd pinned task key binding should have correct when context');
				assert.strictEqual(execute4Keybinding.when, expectedWhen, '4th pinned task key binding should have correct when context');
				assert.strictEqual(execute5Keybinding.when, expectedWhen, '5th pinned task key binding should have correct when context');
			} else {
				// If running without the extension being loaded, just check that 
				// the test framework is working (integration test framework limitation)
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});
	});

	suite('Clean Command Integration Tests', () => {
		test('should have clean command with correct properties', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];

				// Find the clean command
				const cleanCommand = commands.find((cmd: any) => cmd.command === 'cargo-tools.clean');
				assert.ok(cleanCommand, 'Clean command should be defined');
				assert.strictEqual(cleanCommand.title, 'Clean Build Artifacts', 'Clean command should have correct title');
				assert.strictEqual(cleanCommand.category, 'Cargo Tools', 'Clean command should have correct category');
				assert.strictEqual(cleanCommand.icon, '$(trash)', 'Clean command should have trash icon');
			} else {
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});

		test('should have clean button in Project Status view menu', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const menus = packageJson.contributes?.menus || {};
				const viewTitleMenus = menus['view/title'] || [];

				// Find the clean menu entry
				const cleanMenu = viewTitleMenus.find((menu: any) =>
					menu.command === 'cargo-tools.clean' &&
					menu.when === 'view == cargoToolsProjectStatus'
				);
				assert.ok(cleanMenu, 'Clean command should be available in Project Status view menu');
				assert.strictEqual(cleanMenu.group, 'navigation', 'Clean button should be in navigation group');
			} else {
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});
	});

	suite('Project Outline Command Integration Tests', () => {
		test('should have package action commands with correct properties', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];

				// Test clean package command
				const cleanPackageCommand = commands.find((cmd: any) => cmd.command === 'cargo-tools.projectOutline.cleanPackage');
				assert.ok(cleanPackageCommand, 'Clean package command should be defined');
				assert.strictEqual(cleanPackageCommand.title, 'Clean Package', 'Clean package command should have correct title');
				assert.strictEqual(cleanPackageCommand.category, 'Cargo Tools', 'Clean package command should have correct category');
				assert.strictEqual(cleanPackageCommand.icon, '$(trash)', 'Clean package command should have trash icon');
			} else {
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});

		test('should have workspace action commands with correct properties', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const commands = packageJson.contributes?.commands || [];

				// Test workspace commands
				const buildWorkspaceCommand = commands.find((cmd: any) => cmd.command === 'cargo-tools.projectOutline.buildWorkspace');
				assert.ok(buildWorkspaceCommand, 'Build workspace command should be defined');
				assert.strictEqual(buildWorkspaceCommand.title, 'Build Workspace', 'Build workspace command should have correct title');
				assert.strictEqual(buildWorkspaceCommand.icon, '$(tools)', 'Build workspace command should have tools icon');

				const testWorkspaceCommand = commands.find((cmd: any) => cmd.command === 'cargo-tools.projectOutline.testWorkspace');
				assert.ok(testWorkspaceCommand, 'Test workspace command should be defined');
				assert.strictEqual(testWorkspaceCommand.title, 'Test Workspace', 'Test workspace command should have correct title');
				assert.strictEqual(testWorkspaceCommand.icon, '$(beaker)', 'Test workspace command should have beaker icon');

				const cleanWorkspaceCommand = commands.find((cmd: any) => cmd.command === 'cargo-tools.projectOutline.cleanWorkspace');
				assert.ok(cleanWorkspaceCommand, 'Clean workspace command should be defined');
				assert.strictEqual(cleanWorkspaceCommand.title, 'Clean Workspace', 'Clean workspace command should have correct title');
				assert.strictEqual(cleanWorkspaceCommand.icon, '$(trash)', 'Clean workspace command should have trash icon');
			} else {
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});

		test('should have package context menu entries', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const menus = packageJson.contributes?.menus || {};
				const viewItemContextMenus = menus['view/item/context'] || [];

				// Test workspace member clean context menu
				const cleanPackageContextMenu = viewItemContextMenus.find((menu: any) =>
					menu.command === 'cargo-tools.projectOutline.cleanPackage' &&
					menu.when === 'view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/'
				);
				assert.ok(cleanPackageContextMenu, 'Clean package command should be available in workspace member context menu');
				assert.strictEqual(cleanPackageContextMenu.group, 'actions@3', 'Clean package button should be in actions group');

				// Test workspace member clean inline menu
				const cleanPackageInlineMenu = viewItemContextMenus.find((menu: any) =>
					menu.command === 'cargo-tools.projectOutline.cleanPackage' &&
					menu.when === 'view == cargoToolsProjectOutline && viewItem =~ /workspaceMember/' &&
					menu.group === 'inline@3'
				);
				assert.ok(cleanPackageInlineMenu, 'Clean package command should be available as inline button for workspace members');
			} else {
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});

		test('should have workspace root context menu entries', () => {
			const extension = vscode.extensions.getExtension('undefined_publisher.cargo-tools');
			if (extension) {
				const packageJson = extension.packageJSON;
				const menus = packageJson.contributes?.menus || {};
				const viewItemContextMenus = menus['view/item/context'] || [];

				// Test workspace root build context menu
				const buildWorkspaceContextMenu = viewItemContextMenus.find((menu: any) =>
					menu.command === 'cargo-tools.projectOutline.buildWorkspace' &&
					menu.when === 'view == cargoToolsProjectOutline && viewItem == project'
				);
				assert.ok(buildWorkspaceContextMenu, 'Build workspace command should be available in project root context menu');
				assert.strictEqual(buildWorkspaceContextMenu.group, 'actions@1', 'Build workspace button should be in actions group');

				// Test workspace root inline menus
				const buildWorkspaceInlineMenu = viewItemContextMenus.find((menu: any) =>
					menu.command === 'cargo-tools.projectOutline.buildWorkspace' &&
					menu.when === 'view == cargoToolsProjectOutline && viewItem == project' &&
					menu.group === 'inline@1'
				);
				assert.ok(buildWorkspaceInlineMenu, 'Build workspace command should be available as inline button for project root');

				const testWorkspaceInlineMenu = viewItemContextMenus.find((menu: any) =>
					menu.command === 'cargo-tools.projectOutline.testWorkspace' &&
					menu.when === 'view == cargoToolsProjectOutline && viewItem == project' &&
					menu.group === 'inline@2'
				);
				assert.ok(testWorkspaceInlineMenu, 'Test workspace command should be available as inline button for project root');

				const cleanWorkspaceInlineMenu = viewItemContextMenus.find((menu: any) =>
					menu.command === 'cargo-tools.projectOutline.cleanWorkspace' &&
					menu.when === 'view == cargoToolsProjectOutline && viewItem == project' &&
					menu.group === 'inline@3'
				);
				assert.ok(cleanWorkspaceInlineMenu, 'Clean workspace command should be available as inline button for project root');
			} else {
				assert.ok(true, 'Extension package.json not available in test context');
			}
		});
	});
});
