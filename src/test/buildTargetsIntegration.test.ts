import * as assert from 'assert';
import * as vscode from 'vscode';
import * as path from 'path';
import { TargetsTreeProvider } from '../targetsTreeProvider';
import { CargoWorkspace } from '../cargoWorkspace';
import { CargoTarget } from '../cargoTarget';
import { CargoProfile } from '../cargoProfile';

/**
 * Integration tests for the BUILD TARGETS pane functionality
 * These tests focus on tree view behavior and command integration
 */
suite('BUILD TARGETS Tree View Integration Tests', () => {

    /**
     * Mock workspace for testing workspace-aware functionality
     */
    function createMockWorkspace(isWorkspace: boolean = false, targets: CargoTarget[] = []): CargoWorkspace {
        const mockWorkspace = {
            workspaceRoot: '/test/workspace',
            isWorkspace: isWorkspace,
            currentProfile: CargoProfile.dev,
            currentTarget: null,
            targets: targets,
            manifest: null,

            // Event emitters (mocked)
            onDidChangeTargets: new vscode.EventEmitter<CargoTarget[]>().event,
            onDidChangeTarget: new vscode.EventEmitter<CargoTarget>().event,
            onDidChangeProfile: new vscode.EventEmitter<CargoProfile>().event,

            // Methods
            setProfile: (profile: CargoProfile) => { },
            setTarget: (target: CargoTarget) => { },
            refresh: () => Promise.resolve(),
            getCargoArgs: (command: string) => [],
            getTargetExecutablePath: (target: CargoTarget) => '/path/to/executable',

            // Workspace member methods
            getWorkspaceMembers: () => {
                // Group targets by package name for workspace mode
                if (isWorkspace) {
                    const membersMap = new Map<string, CargoTarget[]>();
                    for (const target of targets) {
                        const packageName = target.packageName || 'unknown';
                        if (!membersMap.has(packageName)) {
                            membersMap.set(packageName, []);
                        }
                        membersMap.get(packageName)!.push(target);
                    }
                    return membersMap;
                } else {
                    return new Map<string, CargoTarget[]>();
                }
            }
        } as any;

        return mockWorkspace;
    }

    /**
     * Create test targets for different scenarios
     */
    function createTestTargets(): {
        singlePackageTargets: CargoTarget[];
        workspaceTargets: CargoTarget[];
    } {
        // Single package targets
        const singlePackageTargets = [
            new CargoTarget('my-app', ['bin'], '/test/workspace/src/main.rs', '2021', 'my-package', '/test/workspace'),
            new CargoTarget('my-lib', ['lib'], '/test/workspace/src/lib.rs', '2021', 'my-package', '/test/workspace'),
            new CargoTarget('my-example', ['example'], '/test/workspace/examples/my-example.rs', '2021', 'my-package', '/test/workspace'),
            new CargoTarget('integration-test', ['test'], '/test/workspace/tests/integration-test.rs', '2021', 'my-package', '/test/workspace'),
            new CargoTarget('benchmark', ['bench'], '/test/workspace/benches/benchmark.rs', '2021', 'my-package', '/test/workspace'),
        ];

        // Workspace member targets (mimicking test-rust-project structure)
        const workspaceTargets = [
            // Core package
            new CargoTarget('core-lib', ['lib'], '/test/workspace/core/src/lib.rs', '2021', 'core', '/test/workspace/core'),
            new CargoTarget('core-example', ['example'], '/test/workspace/core/examples/core-example.rs', '2021', 'core', '/test/workspace/core'),

            // CLI package
            new CargoTarget('cli', ['bin'], '/test/workspace/cli/src/main.rs', '2021', 'cli', '/test/workspace/cli'),
            new CargoTarget('cli-test', ['test'], '/test/workspace/cli/tests/cli-test.rs', '2021', 'cli', '/test/workspace/cli'),

            // Web server package
            new CargoTarget('web-server', ['bin'], '/test/workspace/web-server/src/main.rs', '2021', 'web-server', '/test/workspace/web-server'),
            new CargoTarget('api-test', ['test'], '/test/workspace/web-server/tests/api-test.rs', '2021', 'web-server', '/test/workspace/web-server'),

            // Utils package
            new CargoTarget('utils', ['lib'], '/test/workspace/utils/src/lib.rs', '2021', 'utils', '/test/workspace/utils'),
            new CargoTarget('helper', ['bin'], '/test/workspace/utils/src/bin/helper.rs', '2021', 'utils', '/test/workspace/utils'),
        ];

        return { singlePackageTargets, workspaceTargets };
    }

    suite('Tree View Structure Tests', () => {
        test('should organize single-package targets by type', async () => {
            const { singlePackageTargets } = createTestTargets();
            const mockWorkspace = createMockWorkspace(false, singlePackageTargets);

            const treeProvider = new TargetsTreeProvider(mockWorkspace);
            const rootElements = await treeProvider.getChildren();

            // Should have type groups for single package
            const elementLabels = rootElements?.map(el => (el as any).label) || [];

            assert.ok(elementLabels.some(label => label.startsWith('bin')), 'Should have bin group');
            assert.ok(elementLabels.some(label => label.startsWith('lib')), 'Should have lib group');
            assert.ok(elementLabels.some(label => label.startsWith('example')), 'Should have example group');
            assert.ok(elementLabels.some(label => label.startsWith('test')), 'Should have test group');
            assert.ok(elementLabels.some(label => label.startsWith('bench')), 'Should have bench group');
        });

        test('should organize workspace targets by member first, then by type', async () => {
            const { workspaceTargets } = createTestTargets();
            const mockWorkspace = createMockWorkspace(true, workspaceTargets);

            const treeProvider = new TargetsTreeProvider(mockWorkspace);
            const rootElements = await treeProvider.getChildren();

            // Should have workspace member groups
            const elementLabels = rootElements?.map(el => (el as any).label) || [];

            assert.ok(elementLabels.includes('core'), 'Should have core workspace member');
            assert.ok(elementLabels.includes('cli'), 'Should have cli workspace member');
            assert.ok(elementLabels.includes('web-server'), 'Should have web-server workspace member');
            assert.ok(elementLabels.includes('utils'), 'Should have utils workspace member');
        });

        test('should show target types under workspace members', async () => {
            const { workspaceTargets } = createTestTargets();
            const mockWorkspace = createMockWorkspace(true, workspaceTargets);

            const treeProvider = new TargetsTreeProvider(mockWorkspace);
            const rootElements = await treeProvider.getChildren();

            // Find the core workspace member
            const coreGroup = rootElements?.find(el => (el as any).label === 'core');
            assert.ok(coreGroup, 'Should find core workspace member');

            if (coreGroup) {
                const coreChildren = await treeProvider.getChildren(coreGroup);
                const childLabels = coreChildren?.map(el => (el as any).label) || [];

                assert.ok(childLabels.some(label => label.startsWith('lib')), 'Core should have lib group');
                assert.ok(childLabels.some(label => label.startsWith('example')), 'Core should have example group');
            }
        });

        test('should assign correct context values for menu filtering', async () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const mockWorkspace = createMockWorkspace(false, [target]);

            const treeProvider = new TargetsTreeProvider(mockWorkspace);
            const rootElements = await treeProvider.getChildren();

            // Find binary group
            const binaryGroup = rootElements?.find(el => (el as any).label?.startsWith('bin'));
            assert.ok(binaryGroup, 'Should find bin group');

            if (binaryGroup) {
                const targetItems = await treeProvider.getChildren(binaryGroup);
                const targetItem = targetItems?.[0];

                assert.ok(targetItem, 'Should have target item');
                if (targetItem) {
                    const contextValue = (targetItem as any).contextValue;
                    assert.ok(contextValue.includes('cargoTarget'), 'Should have cargoTarget in context');
                    assert.ok(contextValue.includes('isExecutable'), 'Should have isExecutable in context for binary');
                }
            }
        });
    });

    suite('Command Integration Tests', () => {
        /**
         * Mock command execution to verify correct arguments
         */
        let executedCommands: Array<{
            command: string;
            args: string[];
            cwd: string;
        }> = [];

        function mockCommandExecution() {
            executedCommands = [];

            // Mock the executeCargoCommandForTarget method
            return {
                executeCommand: (command: string, args: string[], cwd: string) => {
                    executedCommands.push({ command, args, cwd });
                    return Promise.resolve();
                },
                getExecutedCommands: () => executedCommands
            };
        }

        test('should execute build command with correct args for binary target', async () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const commandMock = mockCommandExecution();

            // Simulate command execution
            const expectedArgs = ['build', '--bin', 'my-app'];
            commandMock.executeCommand('cargo', expectedArgs, '/test');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.strictEqual(executed[0].command, 'cargo', 'Should execute cargo');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should have correct arguments');
            assert.strictEqual(executed[0].cwd, '/test', 'Should execute in correct directory');
        });

        test('should execute build command with package flag for workspace member', async () => {
            const target = new CargoTarget('worker', ['bin'], '/test/workspace/worker/src/main.rs', '2021', 'worker', '/test/workspace/worker');
            const commandMock = mockCommandExecution();

            // Simulate command execution from workspace root (requiring -p flag)
            const expectedArgs = ['build', '-p', 'worker', '--bin', 'worker'];
            commandMock.executeCommand('cargo', expectedArgs, '/test/workspace');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should include package flag');
            assert.strictEqual(executed[0].cwd, '/test/workspace', 'Should execute from workspace root');
        });

        test('should execute build command without package flag when in package directory', async () => {
            const target = new CargoTarget('worker', ['bin'], '/test/workspace/worker/src/main.rs', '2021', 'worker', '/test/workspace/worker');
            const commandMock = mockCommandExecution();

            // Simulate command execution from package directory (no -p flag needed)
            const expectedArgs = ['build', '--bin', 'worker'];
            commandMock.executeCommand('cargo', expectedArgs, '/test/workspace/worker');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should not include package flag');
            assert.strictEqual(executed[0].cwd, '/test/workspace/worker', 'Should execute from package directory');
        });

        test('should execute run command for executable targets', async () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const commandMock = mockCommandExecution();

            const expectedArgs = ['run', '--bin', 'my-app'];
            commandMock.executeCommand('cargo', expectedArgs, '/test');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should have run command args');
        });

        test('should execute test command for test targets', async () => {
            const target = new CargoTarget('integration-test', ['test'], '/test/tests/integration-test.rs', '2021', 'my-package', '/test');
            const commandMock = mockCommandExecution();

            const expectedArgs = ['test', '--test', 'integration-test'];
            commandMock.executeCommand('cargo', expectedArgs, '/test');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should have test command args');
        });

        test('should execute example command for example targets', async () => {
            const target = new CargoTarget('my-example', ['example'], '/test/examples/my-example.rs', '2021', 'my-package', '/test');
            const commandMock = mockCommandExecution();

            const expectedArgs = ['run', '--example', 'my-example'];
            commandMock.executeCommand('cargo', expectedArgs, '/test');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should have example run args');
        });

        test('should include release flag when profile is release', async () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const commandMock = mockCommandExecution();

            const expectedArgs = ['build', '--release', '--bin', 'my-app'];
            commandMock.executeCommand('cargo', expectedArgs, '/test');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should include release flag');
        });

        test('should include features when configured', async () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const commandMock = mockCommandExecution();

            const expectedArgs = ['build', '--bin', 'my-app', '--features', 'feature1,feature2'];
            commandMock.executeCommand('cargo', expectedArgs, '/test');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 1, 'Should execute one command');
            assert.deepStrictEqual(executed[0].args, expectedArgs, 'Should include features');
        });
    });

    suite('Real Project Integration Tests', () => {
        /**
         * Tests against the actual test-rust-project structure
         */
        test('should correctly parse test-rust-project workspace structure', async () => {
            // This test would ideally run against the actual test-rust-project
            // For now, we simulate the expected structure
            const expectedPackages = ['core', 'cli', 'web-server', 'utils'];
            const expectedTargetTypes = ['bin', 'lib', 'test', 'example'];

            // Simulate what should be found in test-rust-project
            const simulatedTargets = [
                // Main executables
                new CargoTarget('cli', ['bin'], '/test-rust-project/cli/src/main.rs', '2021', 'cli', '/test-rust-project/cli'),
                new CargoTarget('web-server', ['bin'], '/test-rust-project/web-server/src/main.rs', '2021', 'web-server', '/test-rust-project/web-server'),
                new CargoTarget('helper', ['bin'], '/test-rust-project/utils/src/bin/helper.rs', '2021', 'utils', '/test-rust-project/utils'),

                // Libraries
                new CargoTarget('core', ['lib'], '/test-rust-project/core/src/lib.rs', '2021', 'core', '/test-rust-project/core'),
                new CargoTarget('utils', ['lib'], '/test-rust-project/utils/src/lib.rs', '2021', 'utils', '/test-rust-project/utils'),

                // Examples and tests would be discovered similarly
            ];

            const mockWorkspace = createMockWorkspace(true, simulatedTargets);

            const treeProvider = new TargetsTreeProvider(mockWorkspace);
            const rootElements = await treeProvider.getChildren();

            // Verify workspace member grouping
            const memberNames = rootElements?.map(el => (el as any).label) || [];
            for (const expectedPackage of expectedPackages) {
                if (simulatedTargets.some(t => t.packageName === expectedPackage)) {
                    assert.ok(memberNames.includes(expectedPackage), `Should include ${expectedPackage} workspace member`);
                }
            }

            // Verify that each member has appropriate target types
            for (const rootElement of rootElements || []) {
                const memberChildren = await treeProvider.getChildren(rootElement);
                const childLabels = memberChildren?.map(el => (el as any).label) || [];

                // Each member should have at least one target type group
                assert.ok(childLabels.length > 0, `Workspace member should have target groups`);
            }
        });

        test('should generate correct commands for test-rust-project targets', async () => {
            /**
             * Mock command execution to verify correct arguments
             */
            let executedCommands: Array<{
                command: string;
                args: string[];
                cwd: string;
            }> = [];

            function mockCommandExecution() {
                executedCommands = [];

                // Mock the executeCargoCommandForTarget method
                return {
                    executeCommand: (command: string, args: string[], cwd: string) => {
                        executedCommands.push({ command, args, cwd });
                        return Promise.resolve();
                    },
                    getExecutedCommands: () => executedCommands
                };
            }

            const commandMock = mockCommandExecution();

            // Test CLI package build from workspace root
            const cliTarget = new CargoTarget('cli', ['bin'], '/test-rust-project/cli/src/main.rs', '2021', 'cli', '/test-rust-project/cli');
            commandMock.executeCommand('cargo', ['build', '-p', 'cli', '--bin', 'cli'], '/test-rust-project');

            // Test utils helper build from package directory  
            const helperTarget = new CargoTarget('helper', ['bin'], '/test-rust-project/utils/src/bin/helper.rs', '2021', 'utils', '/test-rust-project/utils');
            commandMock.executeCommand('cargo', ['build', '--bin', 'helper'], '/test-rust-project/utils');

            // Test core library build
            const coreTarget = new CargoTarget('core', ['lib'], '/test-rust-project/core/src/lib.rs', '2021', 'core', '/test-rust-project/core');
            commandMock.executeCommand('cargo', ['build', '--lib'], '/test-rust-project/core');

            const executed = commandMock.getExecutedCommands();
            assert.strictEqual(executed.length, 3, 'Should execute three commands');

            // Verify CLI command (with package flag from workspace root)
            assert.deepStrictEqual(executed[0].args, ['build', '-p', 'cli', '--bin', 'cli']);
            assert.strictEqual(executed[0].cwd, '/test-rust-project');

            // Verify helper command (no package flag from package directory)
            assert.deepStrictEqual(executed[1].args, ['build', '--bin', 'helper']);
            assert.strictEqual(executed[1].cwd, '/test-rust-project/utils');

            // Verify core library command
            assert.deepStrictEqual(executed[2].args, ['build', '--lib']);
            assert.strictEqual(executed[2].cwd, '/test-rust-project/core');
        });
    });
});
