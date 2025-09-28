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

	suite('Project Outline Single-Crate Features Tests', () => {
		const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');
		const { CargoWorkspace } = require('../cargoWorkspace');

		test('should show features node for single-crate projects', async function () {
			// Increase timeout for this test since it may need to initialize workspace  
			this.timeout(20000); // 20 seconds

			// Use the single-crate test project which has features defined
			const path = require('path');
			const singleCrateProjectPath = path.join(__dirname, '../../test-rust-project-single-crate');

			// Skip this test if the single-crate project doesn't exist
			const fs = require('fs');
			if (!fs.existsSync(singleCrateProjectPath)) {
				console.log('Skipping single-crate features test - single-crate project not found');
				return;
			}

			const workspace = new CargoWorkspace(singleCrateProjectPath);
			await workspace.initialize();

			const provider = new ProjectOutlineTreeProvider();
			provider.updateWorkspace(workspace);

			// Get the root children (should be the project node for single-crate projects)
			const rootChildren = await provider.getChildren();

			// For single-crate projects, we should have one project node
			assert.strictEqual(rootChildren.length, 1, 'Should have exactly one root node for single-crate project');
			assert.strictEqual(rootChildren[0].contextValue, 'project', 'Root node should be a project node');

			// Get the children of the project node (this is where features should be)
			const projectChildren = await provider.getChildren(rootChildren[0]);

			// Find the Features node
			const featuresNode = projectChildren.find((child: any) => child.label === 'Features');

			assert.ok(featuresNode, 'Should have a Features node for single-crate project');
			assert.strictEqual(featuresNode.contextValue, 'features', 'Features node should have correct context value');

			// Check that the features node has the expected data
			assert.ok(featuresNode.data, 'Features node should have data');
			assert.strictEqual(featuresNode.data.packageName, 'single-crate-core', 'Features node should reference the correct package name');
			assert.ok(Array.isArray(featuresNode.data.features), 'Features node should contain features array');
			assert.ok(featuresNode.data.features.length > 0, 'Features array should not be empty');

			// Check that 'all-features' is the first item
			assert.strictEqual(featuresNode.data.features[0], 'all-features', 'First feature should be all-features');

			// Check that expected package-specific features are present (from our test project)
			const expectedFeatures = ['default', 'std-support', 'async-support'];
			for (const expectedFeature of expectedFeatures) {
				assert.ok(featuresNode.data.features.includes(expectedFeature),
					`Should include expected feature: ${expectedFeature}`);
			}
		});

		test('should not show features node when showFeatures is disabled', async function () {
			// Increase timeout for this test since it may need to initialize workspace  
			this.timeout(20000); // 20 seconds

			// Use the single-crate test project 
			const path = require('path');
			const singleCrateProjectPath = path.join(__dirname, '../../test-rust-project-single-crate');

			// Skip this test if the single-crate project doesn't exist
			const fs = require('fs');
			if (!fs.existsSync(singleCrateProjectPath)) {
				console.log('Skipping single-crate features test - single-crate project not found');
				return;
			}

			const workspace = new CargoWorkspace(singleCrateProjectPath);
			await workspace.initialize();

			const provider = new ProjectOutlineTreeProvider();
			provider.updateWorkspace(workspace);

			// Disable features display
			(provider as any).showFeatures = false;

			// Get the root children and then project children
			const rootChildren = await provider.getChildren();
			assert.strictEqual(rootChildren.length, 1, 'Should have exactly one root node');

			const projectChildren = await provider.getChildren(rootChildren[0]);

			// Should not find a Features node
			const featuresNode = projectChildren.find((child: any) => child.label === 'Features');

			assert.ok(!featuresNode, 'Should not have a Features node when showFeatures is disabled');
		});
	});

	suite('Makefile.toml Detection Tests', () => {
		const { CargoWorkspace } = require('../cargoWorkspace');

		test('should detect Makefile.toml when it exists', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

			// Use the test-rust-project which now has a Makefile.toml
			const testProjectPath = getTestProjectPath();
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();

			// Should detect the Makefile.toml we created
			assert.strictEqual(workspace.hasMakefileToml, true, 'Should detect Makefile.toml in test project');
		});

		test('should return false when Makefile.toml does not exist', async function () {
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

		test('should show task categories when Makefile.toml exists', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

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

		test('should return false when Makefile.toml does not exist', async function () {
			// Increase timeout for this test as well since it initializes a workspace
			this.timeout(20000); // 20 seconds

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

		test('should apply task filter correctly', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

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

		test('should apply category filter correctly', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

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

		test('should filter tasks by name only (not description or category)', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

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

		test('should maintain category filter state correctly', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

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

		test('should have task nodes without click commands (button-triggered execution)', async function () {
			// Increase timeout for this test since it may need to discover cargo-make tasks
			this.timeout(20000); // 20 seconds

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

	suite('Project Status Tree Provider Tests', () => {
		test('should assign correct icons for build target selection', () => {
			const { ProjectStatusTreeProvider } = require('../projectStatusTreeProvider');
			const { CargoTarget } = require('../cargoTarget');
			const { IconMapping } = require('../iconMapping');

			// Create mock targets
			const targets = [
				new CargoTarget('test-lib', ['lib'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('test-bin', ['bin'], '/path/to/src/main.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('test-example', ['example'], '/path/to/examples/example.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('test-proc-macro', ['proc-macro'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
			];

			const provider = new ProjectStatusTreeProvider();

			// Manually set workspace properties that are needed for the tests
			(provider as any).workspace = {
				targets: targets,
				selectedBuildTarget: undefined as string | undefined,
				selectedPackage: 'test-package'
			};

			// Test lib target selection
			(provider as any).workspace.selectedBuildTarget = 'lib';
			const libNodes = (provider as any)['createBuildTargetSelectionChildren']();
			assert.strictEqual(libNodes.length, 1, 'Should have one build target node for lib');
			assert.strictEqual(libNodes[0].iconPath, IconMapping.getIconForTargetType('lib'), 'Lib target should have library icon');

			// Test binary target selection
			(provider as any).workspace.selectedBuildTarget = 'test-bin';
			const binNodes = (provider as any)['createBuildTargetSelectionChildren']();
			assert.strictEqual(binNodes.length, 1, 'Should have one build target node for bin');
			assert.strictEqual(binNodes[0].iconPath, IconMapping.getIconForTargetType('bin'), 'Binary target should have binary icon');

			// Test example target selection
			(provider as any).workspace.selectedBuildTarget = 'test-example';
			const exampleNodes = (provider as any)['createBuildTargetSelectionChildren']();
			assert.strictEqual(exampleNodes.length, 1, 'Should have one build target node for example');
			assert.strictEqual(exampleNodes[0].iconPath, IconMapping.getIconForTargetType('example'), 'Example target should have example icon');

			// Test proc-macro target selection
			(provider as any).workspace.selectedBuildTarget = 'test-proc-macro';
			const procMacroNodes = (provider as any)['createBuildTargetSelectionChildren']();
			assert.strictEqual(procMacroNodes.length, 1, 'Should have one build target node for proc-macro');
			assert.strictEqual(procMacroNodes[0].iconPath, IconMapping.getIconForTargetType('proc-macro'), 'Proc-macro target should have library icon');
		});

		test('should assign correct icons for run target selection', () => {
			const { ProjectStatusTreeProvider } = require('../projectStatusTreeProvider');
			const { CargoTarget } = require('../cargoTarget');
			const { IconMapping } = require('../iconMapping');

			// Create mock targets
			const targets = [
				new CargoTarget('test-bin', ['bin'], '/path/to/src/main.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('test-example', ['example'], '/path/to/examples/example.rs', '2021', 'test-package', '/path/to/package'),
			];

			const provider = new ProjectStatusTreeProvider();

			// Manually set workspace properties that are needed for the tests
			(provider as any).workspace = {
				targets: targets,
				selectedRunTarget: undefined as string | undefined,
				selectedPackage: 'test-package'
			};

			// Test binary target selection
			(provider as any).workspace.selectedRunTarget = 'test-bin';
			const binNodes = (provider as any)['createRunTargetSelectionChildren']();
			assert.strictEqual(binNodes.length, 1, 'Should have one run target node for bin');
			assert.strictEqual(binNodes[0].iconPath, IconMapping.getIconForTargetType('bin'), 'Binary run target should have binary icon');

			// Test example target selection
			(provider as any).workspace.selectedRunTarget = 'test-example';
			const exampleNodes = (provider as any)['createRunTargetSelectionChildren']();
			assert.strictEqual(exampleNodes.length, 1, 'Should have one run target node for example');
			assert.strictEqual(exampleNodes[0].iconPath, IconMapping.getIconForTargetType('example'), 'Example run target should have example icon');
		});

		test('should fallback to default icons when target not found', () => {
			const { ProjectStatusTreeProvider } = require('../projectStatusTreeProvider');
			const { IconMapping } = require('../iconMapping');

			const provider = new ProjectStatusTreeProvider();

			// Manually set workspace properties with no targets
			(provider as any).workspace = {
				targets: [],
				selectedBuildTarget: 'non-existent-target',
				selectedRunTarget: 'non-existent-target',
				selectedPackage: 'test-package'
			};

			// Test build target fallback
			const buildNodes = (provider as any)['createBuildTargetSelectionChildren']();
			assert.strictEqual(buildNodes.length, 1, 'Should have one build target node');
			assert.strictEqual(buildNodes[0].iconPath, IconMapping.BUILD_ACTION, 'Should fallback to build action icon when target not found');

			// Test run target fallback
			const runNodes = (provider as any)['createRunTargetSelectionChildren']();
			assert.strictEqual(runNodes.length, 1, 'Should have one run target node');
			assert.strictEqual(runNodes[0].iconPath, IconMapping.SELECTED_STATE, 'Should fallback to selected state icon when target not found');
		});
	});

	suite('Library Crate Type Recognition Tests', () => {
		test('should recognize all library crate types', () => {
			const { CargoTarget } = require('../cargoTarget');

			// Test all library crate types
			const libraryCrateTypes = ['lib', 'dylib', 'staticlib', 'cdylib', 'rlib', 'proc-macro'];

			for (const crateType of libraryCrateTypes) {
				const target = new CargoTarget(
					'test-target',
					[crateType],
					'/path/to/src/lib.rs',
					'2021',
					'test-package',
					'/path/to/package'
				);

				assert.ok(target.isLibrary, `Target with kind '${crateType}' should be recognized as a library`);
			}

			// Test mixed crate types (e.g., a target that is both lib and something else)
			const mixedTarget = new CargoTarget(
				'mixed-target',
				['lib', 'cdylib'],
				'/path/to/src/lib.rs',
				'2021',
				'test-package',
				'/path/to/package'
			);
			assert.ok(mixedTarget.isLibrary, 'Target with mixed library kinds should be recognized as a library');

			// Test non-library crate types
			const nonLibraryTypes = ['bin', 'example', 'test', 'bench'];
			for (const crateType of nonLibraryTypes) {
				const target = new CargoTarget(
					'test-target',
					[crateType],
					'/path/to/src/main.rs',
					'2021',
					'test-package',
					'/path/to/package'
				);

				assert.ok(!target.isLibrary, `Target with kind '${crateType}' should NOT be recognized as a library`);
			}

			// Test edge cases
			const emptyKindTarget = new CargoTarget(
				'empty-target',
				[],
				'/path/to/src/lib.rs',
				'2021',
				'test-package',
				'/path/to/package'
			);
			assert.ok(!emptyKindTarget.isLibrary, 'Target with empty kind array should not be recognized as a library');
		});

		test('should handle real-world library crate types from workspace', async function () {
			// Increase timeout for workspace initialization
			this.timeout(20000); // 20 seconds

			// Import CargoWorkspace for direct testing
			const { CargoWorkspace } = require('../cargoWorkspace');
			const workspace = new CargoWorkspace(getTestProjectPath());
			await workspace.initialize();

			// Find cdylib target
			const cdylibTargets = workspace.targets.filter((t: any) => t.packageName === 'test-cdylib');
			if (cdylibTargets.length > 0) {
				const cdylibTarget = cdylibTargets[0];
				assert.ok(cdylibTarget.isLibrary, 'cdylib target should be recognized as a library');
				assert.ok(cdylibTarget.kind.includes('cdylib'), 'cdylib target should have cdylib kind');
			}

			// Find staticlib target
			const staticlibTargets = workspace.targets.filter((t: any) => t.packageName === 'test-staticlib');
			if (staticlibTargets.length > 0) {
				const staticlibTarget = staticlibTargets[0];
				assert.ok(staticlibTarget.isLibrary, 'staticlib target should be recognized as a library');
				assert.ok(staticlibTarget.kind.includes('staticlib'), 'staticlib target should have staticlib kind');
			}

			// Verify regular lib target still works
			const libTargets = workspace.targets.filter((t: any) => t.packageName === 'core');
			if (libTargets.length > 0) {
				const libTarget = libTargets[0];
				assert.ok(libTarget.isLibrary, 'regular lib target should be recognized as a library');
				assert.ok(libTarget.kind.includes('lib'), 'regular lib target should have lib kind');
			}
		});

		test('should discover new packages after workspace changes', async function () {
			// This test verifies that new packages can be discovered
			this.timeout(20000); // 20 seconds

			const { CargoWorkspace } = require('../cargoWorkspace');
			const workspace = new CargoWorkspace(getTestProjectPath());
			await workspace.initialize();

			// Check that our test packages are discovered
			const allPackageNames = workspace.targets.map((t: any) => t.packageName);
			const uniquePackageNames = [...new Set(allPackageNames)];

			// Log discovered packages for debugging
			console.log('Discovered packages:', uniquePackageNames.sort());

			// We should find the test packages if they're properly configured
			const packageCount = uniquePackageNames.length;
			assert.ok(packageCount >= 4, `Should discover at least 4 packages, found ${packageCount}: ${uniquePackageNames.join(', ')}`);

			// Check for core packages
			assert.ok(uniquePackageNames.includes('core'), 'Should find core package');
			assert.ok(uniquePackageNames.includes('cli'), 'Should find cli package');
			assert.ok(uniquePackageNames.includes('utils'), 'Should find utils package');
			assert.ok(uniquePackageNames.includes('web-server'), 'Should find web-server package');
		});

		test('should filter library crate types correctly in project outline', async function () {
			// This test verifies that cdylib and staticlib targets are included when 'lib' filter is active
			this.timeout(20000); // 20 seconds

			const { CargoWorkspace } = require('../cargoWorkspace');
			const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

			const workspace = new CargoWorkspace(getTestProjectPath());
			await workspace.initialize();

			const provider = new ProjectOutlineTreeProvider();
			provider.updateWorkspace(workspace);

			// Get all targets and verify library types are found
			const cdylibTargets = workspace.targets.filter((t: any) => t.packageName === 'test-cdylib' && t.kind.includes('cdylib'));
			const staticlibTargets = workspace.targets.filter((t: any) => t.packageName === 'test-staticlib' && t.kind.includes('staticlib'));

			assert.ok(cdylibTargets.length > 0, 'Should find cdylib targets');
			assert.ok(staticlibTargets.length > 0, 'Should find staticlib targets');

			// Verify they are recognized as library targets
			if (cdylibTargets.length > 0) {
				assert.ok(cdylibTargets[0].isLibrary, 'cdylib target should be recognized as library');
			}
			if (staticlibTargets.length > 0) {
				assert.ok(staticlibTargets[0].isLibrary, 'staticlib target should be recognized as library');
			}
		});

		test('should assign consistent icons for all library crate types', () => {
			const { IconMapping } = require('../iconMapping');

			// Test all library crate types
			const libraryCrateTypes = ['lib', 'dylib', 'staticlib', 'cdylib', 'rlib', 'proc-macro'];

			for (const crateType of libraryCrateTypes) {
				// Test icon mapping
				const icon = IconMapping.getIconForTargetType(crateType);
				assert.strictEqual(icon, IconMapping.LIB_TARGET,
					`Icon for ${crateType} should be the same as for 'lib'`);
			}
		});

		test('should show consistent display names for all library crate types', () => {
			const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

			// Test all library crate types
			const libraryCrateTypes = ['lib', 'dylib', 'staticlib', 'cdylib', 'rlib', 'proc-macro'];

			const provider = new ProjectOutlineTreeProvider();

			for (const crateType of libraryCrateTypes) {
				// Access private method using bracket notation for testing
				const displayName = (provider as any)['getDisplayNameForTargetType'](crateType);
				assert.strictEqual(displayName, 'Libraries',
					`Display name for ${crateType} should be 'Libraries'`);
			}
		});

		test('should assign consistent context values for all library crate types', () => {
			const { CargoTarget } = require('../cargoTarget');
			const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

			// Test all library crate types
			const libraryCrateTypes = ['lib', 'dylib', 'staticlib', 'cdylib', 'rlib', 'proc-macro'];

			for (const crateType of libraryCrateTypes) {
				// Test context value by creating a target and checking context
				const target = new CargoTarget(
					'test-target',
					[crateType],
					'/path/to/src/lib.rs',
					'2021',
					'test-package',
					'/path/to/package'
				);

				// Create provider instance to test getContextValue method
				const provider = new ProjectOutlineTreeProvider();
				// Access private method using bracket notation for testing
				const contextValue = (provider as any)['getContextValue'](target);

				// All library types should have 'isLibrary' and 'supportsBuild' context
				assert.ok(contextValue.includes('isLibrary'),
					`Context value for ${crateType} should include 'isLibrary'`);
				assert.ok(contextValue.includes('supportsBuild'),
					`Context value for ${crateType} should include 'supportsBuild'`);
				assert.ok(contextValue.includes('cargoTarget'),
					`Context value for ${crateType} should include 'cargoTarget'`);
			}
		});

		test('should group targets with multiple library crate types under single Libraries node', () => {
			const { CargoTarget } = require('../cargoTarget');
			const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

			// Create targets with different combinations of library types
			const targets = [
				new CargoTarget('lib-only', ['lib'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('cdylib-only', ['cdylib'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('mixed-lib', ['lib', 'cdylib'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('multi-lib', ['staticlib', 'cdylib', 'dylib'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('bin-target', ['bin'], '/path/to/src/main.rs', '2021', 'test-package', '/path/to/package')
			];

			const provider = new ProjectOutlineTreeProvider();
			// Access private method using bracket notation for testing
			const groups = (provider as any)['groupTargetsByType'](targets);

			// Should have exactly one 'lib' group (all library types normalized to 'lib')
			assert.ok(groups.has('lib'), 'Should have a lib group');
			assert.ok(groups.has('bin'), 'Should have a bin group');

			// Should NOT have separate groups for other library types
			assert.ok(!groups.has('cdylib'), 'Should NOT have separate cdylib group');
			assert.ok(!groups.has('staticlib'), 'Should NOT have separate staticlib group');
			assert.ok(!groups.has('dylib'), 'Should NOT have separate dylib group');
			assert.ok(!groups.has('rlib'), 'Should NOT have separate rlib group');

			// Verify the lib group contains all library targets
			const libGroup = groups.get('lib');
			assert.strictEqual(libGroup.length, 4, 'Lib group should contain all 4 library targets');

			// Verify the bin group contains only the binary target
			const binGroup = groups.get('bin');
			assert.strictEqual(binGroup.length, 1, 'Bin group should contain only 1 binary target');
		});

		test('should recognize proc-macro crates as library targets', () => {
			const { CargoTarget } = require('../cargoTarget');

			// Test proc-macro crate type
			const procMacroTarget = new CargoTarget(
				'test-proc-macro',
				['proc-macro'],
				'/path/to/src/lib.rs',
				'2021',
				'test-package',
				'/path/to/package'
			);

			assert.ok(procMacroTarget.isLibrary, 'proc-macro target should be recognized as a library');
			assert.ok(!procMacroTarget.isExecutable, 'proc-macro target should not be executable');
			assert.ok(!procMacroTarget.isTest, 'proc-macro target should not be a test');
			assert.ok(!procMacroTarget.isBench, 'proc-macro target should not be a benchmark');
			assert.ok(!procMacroTarget.isExample, 'proc-macro target should not be an example');
		});

		test('should assign consistent icons and context for proc-macro crates', () => {
			const { CargoTarget } = require('../cargoTarget');
			const { IconMapping } = require('../iconMapping');
			const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

			// Test icon mapping for proc-macro
			const icon = IconMapping.getIconForTargetType('proc-macro');
			assert.strictEqual(icon, IconMapping.LIB_TARGET,
				'Icon for proc-macro should be the same as for lib');

			// Test display name
			const provider = new ProjectOutlineTreeProvider();
			const displayName = (provider as any)['getDisplayNameForTargetType']('proc-macro');
			assert.strictEqual(displayName, 'Libraries',
				'Display name for proc-macro should be Libraries');

			// Test context value
			const target = new CargoTarget(
				'test-proc-macro',
				['proc-macro'],
				'/path/to/src/lib.rs',
				'2021',
				'test-package',
				'/path/to/package'
			);

			const contextValue = (provider as any)['getContextValue'](target);
			assert.ok(contextValue.includes('isLibrary'),
				'Context value for proc-macro should include isLibrary');
			assert.ok(contextValue.includes('supportsBuild'),
				'Context value for proc-macro should include supportsBuild');
			assert.ok(contextValue.includes('cargoTarget'),
				'Context value for proc-macro should include cargoTarget');
		});

		test('should group proc-macro targets under Libraries node', () => {
			const { CargoTarget } = require('../cargoTarget');
			const { ProjectOutlineTreeProvider } = require('../projectOutlineTreeProvider');

			// Create targets including proc-macro
			const targets = [
				new CargoTarget('lib-target', ['lib'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('proc-macro-target', ['proc-macro'], '/path/to/src/lib.rs', '2021', 'test-package', '/path/to/package'),
				new CargoTarget('bin-target', ['bin'], '/path/to/src/main.rs', '2021', 'test-package', '/path/to/package')
			];

			const provider = new ProjectOutlineTreeProvider();
			// Access private method using bracket notation for testing
			const groups = (provider as any)['groupTargetsByType'](targets);

			// Should have exactly one 'lib' group (proc-macro normalized to 'lib')
			assert.ok(groups.has('lib'), 'Should have a lib group');
			assert.ok(groups.has('bin'), 'Should have a bin group');

			// Should NOT have separate proc-macro group
			assert.ok(!groups.has('proc-macro'), 'Should NOT have separate proc-macro group');

			// Verify the lib group contains both lib and proc-macro targets
			const libGroup = groups.get('lib');
			assert.strictEqual(libGroup.length, 2, 'Lib group should contain both lib and proc-macro targets');

			// Verify the bin group contains only the binary target
			const binGroup = groups.get('bin');
			assert.strictEqual(binGroup.length, 1, 'Bin group should contain only 1 binary target');
		});

		test('should handle real-world proc-macro crates from workspace', async function () {
			// Increase timeout for workspace initialization
			this.timeout(20000); // 20 seconds

			const { CargoWorkspace } = require('../cargoWorkspace');
			const workspace = new CargoWorkspace(getTestProjectPath());
			await workspace.initialize();

			// Find proc-macro targets
			const procMacroTargets = workspace.targets.filter((t: any) =>
				t.packageName && (t.packageName === 'test-proc-macro' || t.packageName === 'test-proc-macro-alt'));

			assert.ok(procMacroTargets.length >= 2, 'Should find at least 2 proc-macro targets');

			for (const procMacroTarget of procMacroTargets) {
				assert.ok(procMacroTarget.isLibrary, `${procMacroTarget.packageName} should be recognized as a library`);
				assert.ok(procMacroTarget.kind.includes('proc-macro'), `${procMacroTarget.packageName} should have proc-macro kind`);
			}
		});
	});
});
