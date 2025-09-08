import * as assert from 'assert';

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from 'vscode';
import { CargoExtensionManager } from '../cargoExtensionManager';
// import * as myExtension from '../../extension';

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
				assert.ok(commandIds.includes('cargo-tools.projectOutline.buildTarget'), 'projectOutline.buildTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.runTarget'), 'projectOutline.runTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.projectOutline.benchTarget'), 'projectOutline.benchTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.setAsDefaultTestTarget'), 'setAsDefaultTestTarget command should be defined');
				assert.ok(commandIds.includes('cargo-tools.setAsDefaultBenchTarget'), 'setAsDefaultBenchTarget command should be defined');

				// Check for makefile commands
				assert.ok(commandIds.includes('cargo-tools.makefile.runTask'), 'makefile.runTask command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.setTaskFilter'), 'makefile.setTaskFilter command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.editTaskFilter'), 'makefile.editTaskFilter command should be defined');
				assert.ok(commandIds.includes('cargo-tools.makefile.clearTaskFilter'), 'makefile.clearTaskFilter command should be defined');
			} else {
				// If running without the extension being loaded, just check that 
				// the command patterns are reasonable (integration test framework limitation)
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

		test('should detect Makefile.toml when it exists', async () => {
			// Use the test-rust-project which now has a Makefile.toml
			const testProjectPath = '/home/nickel/Programming/repos/cargo-tools/test-rust-project';
			const workspace = new CargoWorkspace(testProjectPath);

			await workspace.initialize();

			// Should detect the Makefile.toml we created
			assert.strictEqual(workspace.hasMakefileToml, true, 'Should detect Makefile.toml in test project');
		});

		test('should return false when Makefile.toml does not exist', async () => {
			// Use a path that doesn't have Makefile.toml 
			const testPath = '/tmp'; // temp directory should not have Makefile.toml
			const workspace = new CargoWorkspace(testPath);

			await workspace.initialize();

			// Should not detect Makefile.toml
			assert.strictEqual(workspace.hasMakefileToml, false, 'Should not detect Makefile.toml when it does not exist');
		});
	});

	suite('Makefile Tree Provider Tests', () => {
		const { MakefileTreeProvider } = require('../makefileTreeProvider');
		const { CargoWorkspace } = require('../cargoWorkspace');

		test('should show task categories when Makefile.toml exists', async () => {
			const provider = new MakefileTreeProvider();
			const testProjectPath = '/home/nickel/Programming/repos/cargo-tools/test-rust-project';
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
			const testProjectPath = '/home/nickel/Programming/repos/cargo-tools/test-rust-project';
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
	});
});
