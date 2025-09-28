import * as assert from 'assert';
import * as vscode from 'vscode';
import { StateManager } from '../stateManager';

// Mock ExtensionContext for testing
class MockExtensionContext implements vscode.ExtensionContext {
    subscriptions: vscode.Disposable[] = [];

    private globalStateData = new Map<string, any>();
    private workspaceStateData = new Map<string, any>();

    workspaceState: vscode.Memento = {
        get: <T>(key: string, defaultValue?: T) => {
            return this.workspaceStateData.get(key) ?? defaultValue;
        },
        update: (key: string, value: any) => {
            this.workspaceStateData.set(key, value);
            return Promise.resolve();
        },
        keys: () => Array.from(this.workspaceStateData.keys())
    };

    globalState: vscode.Memento & { setKeysForSync(keys: string[]): void } = {
        get: <T>(key: string, defaultValue?: T) => {
            return this.globalStateData.get(key) ?? defaultValue;
        },
        update: (key: string, value: any) => {
            this.globalStateData.set(key, value);
            return Promise.resolve();
        },
        keys: () => Array.from(this.globalStateData.keys()),
        setKeysForSync: (keys: string[]) => {
            // Mock implementation
        }
    };

    secrets: vscode.SecretStorage = {} as any;
    extensionUri: vscode.Uri = vscode.Uri.file('/test');
    extensionPath: string = '/test';
    environmentVariableCollection: vscode.GlobalEnvironmentVariableCollection = {} as any;
    extension: vscode.Extension<any> = {} as any;
    languageModelAccessInformation: vscode.LanguageModelAccessInformation = {} as any;

    asAbsolutePath(relativePath: string): string {
        return `/test/${relativePath}`;
    }

    storageUri: vscode.Uri = vscode.Uri.file('/test/storage');
    storagePath: string = '/test/storage';
    globalStorageUri: vscode.Uri = vscode.Uri.file('/test/global');
    globalStoragePath: string = '/test/global';
    logUri: vscode.Uri = vscode.Uri.file('/test/log');
    logPath: string = '/test/log';
    extensionMode: vscode.ExtensionMode = vscode.ExtensionMode.Test;
}

// Mock WorkspaceFolder for testing
const mockWorkspaceFolder: vscode.WorkspaceFolder = {
    uri: vscode.Uri.file('/test/workspace'),
    name: 'test-workspace',
    index: 0
};

suite('State Persistence Tests', () => {
    let stateManager: StateManager;
    let mockContext: MockExtensionContext;

    setup(() => {
        mockContext = new MockExtensionContext();
        stateManager = new StateManager(mockContext, mockWorkspaceFolder);
    });

    suite('Project Status View State', () => {
        test('should persist and retrieve selected package', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const packageName = 'my-package';

            // Set selected package
            await stateManager.setSelectedPackage(folderName, packageName, isMultiProject);

            // Retrieve selected package
            const retrievedPackage = stateManager.getSelectedPackage(folderName, isMultiProject);
            assert.strictEqual(retrievedPackage, packageName);
        });

        test('should persist and retrieve selected build target', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const { CargoTarget, CargoTargetKind } = require('../cargoTarget');
            const target = new CargoTarget('my-binary', CargoTargetKind.Bin, '/path/to/src/main.rs', '2021', 'test-package', '/path/to/package');

            await stateManager.setSelectedBuildTarget(folderName, target, isMultiProject);

            const retrievedTarget = stateManager.getSelectedBuildTarget(folderName, isMultiProject);
            assert.strictEqual(retrievedTarget?.name, target.name);
            assert.strictEqual(retrievedTarget?.kind, target.kind);
        });

        test('should persist and retrieve selected run target', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const { CargoTarget, CargoTargetKind } = require('../cargoTarget');
            const target = new CargoTarget('my-example', CargoTargetKind.Example, '/path/to/examples/example.rs', '2021', 'test-package', '/path/to/package');

            await stateManager.setSelectedRunTarget(folderName, target, isMultiProject);

            const retrievedTarget = stateManager.getSelectedRunTarget(folderName, isMultiProject);
            assert.strictEqual(retrievedTarget?.name, target.name);
            assert.strictEqual(retrievedTarget?.kind, target.kind);
        });

        test('should persist and retrieve selected benchmark target', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const { CargoTarget, CargoTargetKind } = require('../cargoTarget');
            const target = new CargoTarget('my-benchmark', CargoTargetKind.Bench, '/path/to/benches/bench.rs', '2021', 'test-package', '/path/to/package');

            await stateManager.setSelectedBenchmarkTarget(folderName, target, isMultiProject);

            const retrievedTarget = stateManager.getSelectedBenchmarkTarget(folderName, isMultiProject);
            assert.strictEqual(retrievedTarget?.name, target.name);
            assert.strictEqual(retrievedTarget?.kind, target.kind);
        });

        test('should persist and retrieve selected platform target', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const platformTarget = 'x86_64-unknown-linux-gnu';

            await stateManager.setSelectedPlatformTarget(folderName, platformTarget, isMultiProject);

            const retrievedTarget = stateManager.getSelectedPlatformTarget(folderName, isMultiProject);
            assert.strictEqual(retrievedTarget, platformTarget);
        });

        test('should persist and retrieve selected features', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const features = ['feature1', 'feature2', 'feature3'];

            await stateManager.setSelectedFeatures(folderName, features, isMultiProject);

            const retrievedFeatures = stateManager.getSelectedFeatures(folderName, isMultiProject);
            assert.deepStrictEqual(retrievedFeatures, features);
        });

        test('should persist and retrieve selected profile', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const profileName = 'release';

            await stateManager.setSelectedProfile(folderName, profileName, isMultiProject);

            const retrievedProfile = stateManager.getSelectedProfile(folderName, isMultiProject);
            assert.strictEqual(retrievedProfile, profileName);
        });
    });

    suite('Project Outline View State', () => {
        test('should persist and retrieve group by workspace member setting', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const groupBy = false;

            await stateManager.setGroupByWorkspaceMember(folderName, groupBy, isMultiProject);

            const retrievedGroupBy = stateManager.getGroupByWorkspaceMember(folderName, isMultiProject);
            assert.strictEqual(retrievedGroupBy, groupBy);
        });

        test('should persist and retrieve workspace member filter', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const filter = 'core';

            await stateManager.setWorkspaceMemberFilter(folderName, filter, isMultiProject);

            const retrievedFilter = stateManager.getWorkspaceMemberFilter(folderName, isMultiProject);
            assert.strictEqual(retrievedFilter, filter);
        });

        test('should persist and retrieve target type filter', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const targetTypes = ['bin', 'lib'];

            await stateManager.setTargetTypeFilter(folderName, targetTypes, isMultiProject);

            const retrievedTargetTypes = stateManager.getTargetTypeFilter(folderName, isMultiProject);
            assert.deepStrictEqual(retrievedTargetTypes, targetTypes);
        });

        test('should persist and retrieve target type filter active state', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const isActive = true;

            await stateManager.setIsTargetTypeFilterActive(folderName, isActive, isMultiProject);

            const retrievedIsActive = stateManager.getIsTargetTypeFilterActive(folderName, isMultiProject);
            assert.strictEqual(retrievedIsActive, isActive);
        });

        test('should persist and retrieve show features setting', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const showFeatures = false;

            await stateManager.setShowFeatures(folderName, showFeatures, isMultiProject);

            const retrievedShowFeatures = stateManager.getShowFeatures(folderName, isMultiProject);
            assert.strictEqual(retrievedShowFeatures, showFeatures);
        });
    });

    suite('Multi-Project Workspace Support', () => {
        test('should store different state for multi-project workspaces', async () => {
            const folderName = 'test-workspace';
            const packageName1 = 'package-single';
            const packageName2 = 'package-multi';

            // Set state for single project
            await stateManager.setSelectedPackage(folderName, packageName1, false);

            // Set state for multi project
            await stateManager.setSelectedPackage(folderName, packageName2, true);

            // Verify different values are stored
            const singleProjectPackage = stateManager.getSelectedPackage(folderName, false);
            const multiProjectPackage = stateManager.getSelectedPackage(folderName, true);

            assert.strictEqual(singleProjectPackage, packageName1);
            assert.strictEqual(multiProjectPackage, packageName2);
        });

        test('should generate different keys for single vs multi-project', async () => {
            const folderName = 'test-workspace';
            const value = 'test-value';

            await stateManager.setSelectedPackage(folderName, value, false);
            await stateManager.setSelectedPackage(folderName, value, true);

            // Check that different keys are generated in global state
            const keys = mockContext.globalState.keys();
            const singleProjectKey = keys.find(key => key.includes('selectedPackage') && !key.includes(`${folderName} `));
            const multiProjectKey = keys.find(key => key.includes('selectedPackage') && key.includes(`${folderName} `));

            assert.ok(singleProjectKey, 'Single project key should exist');
            assert.ok(multiProjectKey, 'Multi project key should exist');
            assert.notStrictEqual(singleProjectKey, multiProjectKey, 'Keys should be different');
        });
    });

    suite('Default Values', () => {
        test('should return default values when no state is persisted', () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;

            // Test default values for Project Status View
            assert.strictEqual(stateManager.getSelectedPackage(folderName, isMultiProject), undefined);
            assert.strictEqual(stateManager.getSelectedBuildTarget(folderName, isMultiProject), null);
            assert.strictEqual(stateManager.getSelectedRunTarget(folderName, isMultiProject), null);
            assert.strictEqual(stateManager.getSelectedBenchmarkTarget(folderName, isMultiProject), null);
            assert.strictEqual(stateManager.getSelectedPlatformTarget(folderName, isMultiProject), null);
            assert.deepStrictEqual(stateManager.getSelectedFeatures(folderName, isMultiProject), []);
            assert.strictEqual(stateManager.getSelectedProfile(folderName, isMultiProject), null);

            // Test default values for Project Outline View
            assert.strictEqual(stateManager.getGroupByWorkspaceMember(folderName, isMultiProject), true);
            assert.strictEqual(stateManager.getWorkspaceMemberFilter(folderName, isMultiProject), '');
            assert.deepStrictEqual(stateManager.getTargetTypeFilter(folderName, isMultiProject), ['bin', 'lib', 'example', 'bench']);
            assert.strictEqual(stateManager.getIsTargetTypeFilterActive(folderName, isMultiProject), false);
            assert.strictEqual(stateManager.getShowFeatures(folderName, isMultiProject), true);
        });
    });

    suite('Reset Functionality', () => {
        test('should reset all state to defaults', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;

            // Set some non-default state
            const { CargoTarget, CargoTargetKind } = require('../cargoTarget');
            const target = new CargoTarget('test-target', CargoTargetKind.Bin, '/path/to/src/main.rs', '2021', 'test-package', '/path/to/package');

            await stateManager.setSelectedPackage(folderName, 'test-package', isMultiProject);
            await stateManager.setSelectedBuildTarget(folderName, target, isMultiProject);
            await stateManager.setGroupByWorkspaceMember(folderName, false, isMultiProject);
            await stateManager.setWorkspaceMemberFilter(folderName, 'filter', isMultiProject);
            await stateManager.setShowFeatures(folderName, false, isMultiProject);

            // Reset all state
            await stateManager.reset(folderName, isMultiProject);

            // Verify all state is reset to defaults
            assert.strictEqual(stateManager.getSelectedPackage(folderName, isMultiProject), undefined);
            assert.strictEqual(stateManager.getSelectedBuildTarget(folderName, isMultiProject), null);
            assert.strictEqual(stateManager.getGroupByWorkspaceMember(folderName, isMultiProject), true);
            assert.strictEqual(stateManager.getWorkspaceMemberFilter(folderName, isMultiProject), '');
            assert.strictEqual(stateManager.getShowFeatures(folderName, isMultiProject), true);
        });
    });

    suite('Workspace-Unique Keys', () => {
        test('should generate unique keys based on workspace path', async () => {
            const folderName = 'test-workspace';
            const isMultiProject = false;
            const value = 'test-value';

            await stateManager.setSelectedPackage(folderName, value, isMultiProject);

            // Check that the key includes the workspace folder path
            const keys = mockContext.globalState.keys();
            const packageKey = keys.find(key => key.includes('selectedPackage'));

            assert.ok(packageKey, 'Package key should exist');
            assert.ok(packageKey!.includes(mockWorkspaceFolder.uri.fsPath), 'Key should include workspace path');
        });

        test('should create different keys for different workspace folders', () => {
            const mockContext2 = new MockExtensionContext();
            const mockWorkspaceFolder2: vscode.WorkspaceFolder = {
                uri: vscode.Uri.file('/test/workspace2'),
                name: 'test-workspace2',
                index: 1
            };
            const stateManager2 = new StateManager(mockContext2, mockWorkspaceFolder2);

            const folderName = 'test-workspace';
            const isMultiProject = false;

            // Set package in both state managers
            stateManager.setSelectedPackage(folderName, 'package1', isMultiProject);
            stateManager2.setSelectedPackage(folderName, 'package2', isMultiProject);

            // Verify keys are different (they should have different workspace paths)
            const keys1 = mockContext.globalState.keys();
            const keys2 = mockContext2.globalState.keys();

            // The state managers should have generated different keys due to different workspace folder paths
            // Since they use different contexts, they will store in different mock global states
            assert.notDeepStrictEqual(keys1, keys2, 'Different state managers should generate different storage patterns');
        });
    });
});
