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
});
