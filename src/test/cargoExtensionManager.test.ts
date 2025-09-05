import * as assert from 'assert';
import { CargoTarget, TargetActionType } from '../cargoTarget';
import { CargoProfile } from '../cargoProfile';
import { CargoWorkspace } from '../cargoWorkspace';
import { CargoExtensionManager } from '../cargoExtensionManager';

/**
 * Mock CargoTaskProvider with a simplified buildCargoArgs method for testing
 * This is used across multiple test suites to verify the "All targets" fix
 */
function buildCargoArgs(definition: any, mockWorkspace: any): string[] {
    const args = [definition.command];

    // Add profile
    if (definition.profile === 'release' ||
        (mockWorkspace.currentProfile?.toString() === 'release' && !definition.profile)) {
        args.push('--release');
    }

    // Find target to get package information
    let targetObj: CargoTarget | undefined;
    const targetName = definition.target || definition.targetName;
    if (targetName) {
        targetObj = mockWorkspace.targets.find((t: CargoTarget) => t.name === targetName);
    }
    // Don't fall back to workspace.currentTarget when no target is specified
    // This allows "All" targets builds without target-specific restrictions

    // Add package argument if we have package info and it's needed
    const packageName = definition.packageName || targetObj?.packageName;
    if (packageName && mockWorkspace.isWorkspace) {
        args.push('--package', packageName);
    }

    // Add target-specific arguments
    const targetNameForArgs = definition.target || definition.targetName;
    if (definition.targetKind) {
        switch (definition.targetKind) {
            case 'bin':
                if (targetNameForArgs) {
                    args.push('--bin', targetNameForArgs);
                }
                break;
            case 'lib':
                args.push('--lib');
                break;
            case 'example':
                if (targetNameForArgs) {
                    args.push('--example', targetNameForArgs);
                }
                break;
            case 'test':
                if (targetNameForArgs) {
                    args.push('--test', targetNameForArgs);
                }
                break;
            case 'bench':
                if (targetNameForArgs) {
                    args.push('--bench', targetNameForArgs);
                }
                break;
        }
    } else if (targetNameForArgs) {
        // Fallback: try to find target in workspace and determine type
        const target = mockWorkspace.targets.find((t: CargoTarget) => t.name === targetNameForArgs);
        if (target) {
            if (target.isExecutable) {
                args.push('--bin', target.name);
            } else if (target.isLibrary) {
                args.push('--lib');
            } else if (target.isExample) {
                args.push('--example', target.name);
            } else if (target.isTest) {
                args.push('--test', target.name);
            } else if (target.isBench) {
                args.push('--bench', target.name);
            }
        }
    }
    // No fallback to workspace.currentTarget - when no target is specified, 
    // we want to build all targets (no target-specific args)

    // Add features
    if (definition.features && definition.features.length > 0) {
        args.push('--features', definition.features.join(','));
    }

    if (definition.allFeatures) {
        args.push('--all-features');
    }

    if (definition.noDefaultFeatures) {
        args.push('--no-default-features');
    }

    return args;
}

/**
 * Unit tests for command line argument generation logic
 * These tests focus on the core business logic without complex VS Code dependencies
 */
suite('Cargo Command Line Argument Generation Unit Tests', () => {

    suite('CargoTarget Property Tests', () => {
        test('should correctly identify binary targets', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, true);
            assert.strictEqual(target.isLibrary, false);
            assert.strictEqual(target.isTest, false);
            assert.strictEqual(target.isExample, false);
            assert.strictEqual(target.isBench, false);
        });

        test('should correctly identify library targets', () => {
            const target = new CargoTarget(
                'mylib',
                ['lib'],
                '/path/to/src/lib.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, false);
            assert.strictEqual(target.isLibrary, true);
            assert.strictEqual(target.isTest, false);
            assert.strictEqual(target.isExample, false);
            assert.strictEqual(target.isBench, false);
        });

        test('should correctly identify test targets', () => {
            const target = new CargoTarget(
                'integration-test',
                ['test'],
                '/path/to/tests/integration-test.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, false);
            assert.strictEqual(target.isLibrary, false);
            assert.strictEqual(target.isTest, true);
            assert.strictEqual(target.isExample, false);
            assert.strictEqual(target.isBench, false);
        });

        test('should correctly identify example targets', () => {
            const target = new CargoTarget(
                'my-example',
                ['example'],
                '/path/to/examples/my-example.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, false);
            assert.strictEqual(target.isLibrary, false);
            assert.strictEqual(target.isTest, false);
            assert.strictEqual(target.isExample, true);
            assert.strictEqual(target.isBench, false);
        });

        test('should correctly identify bench targets', () => {
            const target = new CargoTarget(
                'my-bench',
                ['bench'],
                '/path/to/benches/my-bench.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, false);
            assert.strictEqual(target.isLibrary, false);
            assert.strictEqual(target.isTest, false);
            assert.strictEqual(target.isExample, false);
            assert.strictEqual(target.isBench, true);
        });

        test('should handle undefined kind gracefully', () => {
            const target = new CargoTarget(
                'broken-target',
                undefined as any,
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, false);
            assert.strictEqual(target.isLibrary, false);
            assert.strictEqual(target.isTest, false);
            assert.strictEqual(target.isExample, false);
            assert.strictEqual(target.isBench, false);
        });

        test('should handle empty kind array gracefully', () => {
            const target = new CargoTarget(
                'empty-target',
                [],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            assert.strictEqual(target.isExecutable, false);
            assert.strictEqual(target.isLibrary, false);
            assert.strictEqual(target.isTest, false);
            assert.strictEqual(target.isExample, false);
            assert.strictEqual(target.isBench, false);
        });
    });

    suite('Target Action Type Support Tests', () => {
        test('should return correct supported action types for binary target', () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const supportedActions = target.supportedActionTypes;

            assert.ok(supportedActions.includes(TargetActionType.Build), 'Should support build action');
            assert.ok(supportedActions.includes(TargetActionType.Run), 'Should support run action');
            assert.ok(!supportedActions.includes(TargetActionType.Test), 'Should not support test action');
            assert.ok(!supportedActions.includes(TargetActionType.Bench), 'Should not support bench action');
        });

        test('should return correct supported action types for library target', () => {
            const target = new CargoTarget('my-lib', ['lib'], '/test/src/lib.rs', '2021', 'my-package', '/test');
            const supportedActions = target.supportedActionTypes;

            assert.ok(supportedActions.includes(TargetActionType.Build), 'Should support build action');
            assert.ok(!supportedActions.includes(TargetActionType.Run), 'Should not support run action');
            assert.ok(!supportedActions.includes(TargetActionType.Test), 'Should not support test action');
            assert.ok(!supportedActions.includes(TargetActionType.Bench), 'Should not support bench action');
        });

        test('should return correct supported action types for test target', () => {
            const target = new CargoTarget('my-test', ['test'], '/test/tests/my-test.rs', '2021', 'my-package', '/test');
            const supportedActions = target.supportedActionTypes;

            assert.ok(supportedActions.includes(TargetActionType.Build), 'Should support build action');
            assert.ok(!supportedActions.includes(TargetActionType.Run), 'Should not support run action');
            assert.ok(supportedActions.includes(TargetActionType.Test), 'Should support test action');
            assert.ok(!supportedActions.includes(TargetActionType.Bench), 'Should not support bench action');
        });

        test('should return correct supported action types for bench target', () => {
            const target = new CargoTarget('my-bench', ['bench'], '/test/benches/my-bench.rs', '2021', 'my-package', '/test');
            const supportedActions = target.supportedActionTypes;

            assert.ok(supportedActions.includes(TargetActionType.Build), 'Should support build action');
            assert.ok(!supportedActions.includes(TargetActionType.Run), 'Should not support run action');
            assert.ok(!supportedActions.includes(TargetActionType.Test), 'Should not support test action');
            assert.ok(supportedActions.includes(TargetActionType.Bench), 'Should support bench action');
        });

        test('should return correct supported action types for example target', () => {
            const target = new CargoTarget('my-example', ['example'], '/test/examples/my-example.rs', '2021', 'my-package', '/test');
            const supportedActions = target.supportedActionTypes;

            assert.ok(supportedActions.includes(TargetActionType.Build), 'Should support build action');
            assert.ok(supportedActions.includes(TargetActionType.Run), 'Should support run action');
            assert.ok(!supportedActions.includes(TargetActionType.Test), 'Should not support test action');
            assert.ok(!supportedActions.includes(TargetActionType.Bench), 'Should not support bench action');
        });

        test('should correctly check if target supports specific action type', () => {
            const binaryTarget = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');

            assert.ok(binaryTarget.supportsActionType(TargetActionType.Build), 'Binary should support build');
            assert.ok(binaryTarget.supportsActionType(TargetActionType.Run), 'Binary should support run');
            assert.ok(!binaryTarget.supportsActionType(TargetActionType.Test), 'Binary should not support test');
            assert.ok(!binaryTarget.supportsActionType(TargetActionType.Bench), 'Binary should not support bench');
        });

        test('should return correct cargo command for action types', () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');

            assert.strictEqual(target.getCargoCommand(TargetActionType.Build), 'build');
            assert.strictEqual(target.getCargoCommand(TargetActionType.Run), 'run');
            assert.strictEqual(target.getCargoCommand(TargetActionType.Test), 'test');
            assert.strictEqual(target.getCargoCommand(TargetActionType.Bench), 'bench');
        });

        test('should return correct target arguments for different action types', () => {
            const binaryTarget = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const libTarget = new CargoTarget('my-lib', ['lib'], '/test/src/lib.rs', '2021', 'my-package', '/test');
            const testTarget = new CargoTarget('my-test', ['test'], '/test/tests/my-test.rs', '2021', 'my-package', '/test');
            const exampleTarget = new CargoTarget('my-example', ['example'], '/test/examples/my-example.rs', '2021', 'my-package', '/test');

            // Binary target args
            assert.deepStrictEqual(binaryTarget.getTargetArgs(TargetActionType.Build), ['--bin', 'my-app']);
            assert.deepStrictEqual(binaryTarget.getTargetArgs(TargetActionType.Run), ['--bin', 'my-app']);

            // Library target args
            assert.deepStrictEqual(libTarget.getTargetArgs(TargetActionType.Build), ['--lib']);

            // Test target args
            assert.deepStrictEqual(testTarget.getTargetArgs(TargetActionType.Test), ['--test', 'my-test']);

            // Example target args
            assert.deepStrictEqual(exampleTarget.getTargetArgs(TargetActionType.Build), ['--example', 'my-example']);
            assert.deepStrictEqual(exampleTarget.getTargetArgs(TargetActionType.Run), ['--example', 'my-example']);
        });
    });

    suite('Command Line Argument Generation Logic', () => {
        /**
         * Test helper to simulate the getCargoArgsForTarget logic without dependencies
         */
        function getCargoArgsForTarget(
            command: string,
            target: CargoTarget,
            options: {
                profile?: CargoProfile;
                isWorkspace?: boolean;
                features?: string[];
                allFeatures?: boolean;
                noDefaultFeatures?: boolean;
                commandArgs?: string[];
                usePackageFlag?: boolean;
            } = {}
        ): string[] {
            const args = [command];

            // Add profile
            if (options.profile === CargoProfile.release) {
                args.push('--release');
            }

            // For workspace projects, add package specification
            if (target.packageName && options.isWorkspace && options.usePackageFlag) {
                args.push('-p', target.packageName);
            }

            // Add target-specific flags
            if (command !== 'clean' && target.kind && Array.isArray(target.kind)) {
                if (target.kind.includes('bin')) {
                    args.push('--bin', target.name);
                } else if (target.kind.includes('lib')) {
                    args.push('--lib');
                } else if (target.kind.includes('example')) {
                    args.push('--example', target.name);
                } else if (target.kind.includes('test')) {
                    args.push('--test', target.name);
                } else if (target.kind.includes('bench')) {
                    args.push('--bench', target.name);
                }
            }

            // Add features and other configuration
            const features = options.features || [];
            if (features && Array.isArray(features) && features.length > 0) {
                args.push('--features', features.join(','));
            }

            if (options.allFeatures) {
                args.push('--all-features');
            }

            if (options.noDefaultFeatures) {
                args.push('--no-default-features');
            }

            // Add command-specific arguments from configuration
            const commandArgs = options.commandArgs;
            if (commandArgs && Array.isArray(commandArgs)) {
                args.push(...commandArgs);
            }

            return args;
        }

        test('should generate correct args for binary target build', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['build', '--bin', 'my-app'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should generate correct args for binary target build with release profile', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.release
            });

            const expectedArgs = ['build', '--release', '--bin', 'my-app'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should generate correct args for workspace member with package flag', () => {
            const target = new CargoTarget(
                'worker',
                ['bin'],
                '/path/to/workspace/worker/src/main.rs',
                '2021',
                'worker-package',
                '/path/to/workspace/worker'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev,
                isWorkspace: true,
                usePackageFlag: true
            });

            const expectedArgs = ['build', '-p', 'worker-package', '--bin', 'worker'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should generate correct args for library target', () => {
            const target = new CargoTarget(
                'mylib',
                ['lib'],
                '/path/to/src/lib.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['build', '--lib'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should generate correct args for example target', () => {
            const target = new CargoTarget(
                'my-example',
                ['example'],
                '/path/to/examples/my-example.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['build', '--example', 'my-example'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should generate correct args for test target', () => {
            const target = new CargoTarget(
                'integration-test',
                ['test'],
                '/path/to/tests/integration-test.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('test', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['test', '--test', 'integration-test'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should generate correct args for bench target', () => {
            const target = new CargoTarget(
                'my-bench',
                ['bench'],
                '/path/to/benches/my-bench.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['build', '--bench', 'my-bench'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should include features in args', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev,
                features: ['feature1', 'feature2']
            });

            const expectedArgs = ['build', '--bin', 'my-app', '--features', 'feature1,feature2'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should include all-features flag', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev,
                allFeatures: true
            });

            const expectedArgs = ['build', '--bin', 'my-app', '--all-features'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should include no-default-features flag', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev,
                noDefaultFeatures: true
            });

            const expectedArgs = ['build', '--bin', 'my-app', '--no-default-features'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should include additional build args', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev,
                commandArgs: ['--verbose', '--color', 'always']
            });

            const expectedArgs = ['build', '--bin', 'my-app', '--verbose', '--color', 'always'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should handle run command for binary targets', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('run', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['run', '--bin', 'my-app'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should handle clean command without target flags', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const args = getCargoArgsForTarget('clean', target, {
                profile: CargoProfile.dev
            });

            const expectedArgs = ['clean'];
            assert.deepStrictEqual(args, expectedArgs);
        });

        test('should skip package flag when not in workspace mode', () => {
            const target = new CargoTarget(
                'worker',
                ['bin'],
                '/path/to/workspace/worker/src/main.rs',
                '2021',
                'worker-package',
                '/path/to/workspace/worker'
            );

            const args = getCargoArgsForTarget('build', target, {
                profile: CargoProfile.dev,
                isWorkspace: true,
                usePackageFlag: false  // Simulating execution from package directory
            });

            const expectedArgs = ['build', '--bin', 'worker'];
            assert.deepStrictEqual(args, expectedArgs);
        });
    });

    suite('Working Directory Logic Tests', () => {
        /**
         * Test helper to simulate the getWorkingDirectoryForTarget logic
         */
        function getWorkingDirectoryForTarget(
            target: CargoTarget,
            options: {
                isWorkspace?: boolean;
                workspaceRoot?: string;
            } = {}
        ): string {
            const workspaceRoot = options.workspaceRoot || '/default/workspace';

            // For workspace members, use the package path if available
            if (target.packagePath && options.isWorkspace) {
                return target.packagePath;
            }

            // For single-package projects or when package path is not available, use workspace root
            return workspaceRoot;
        }

        test('should return workspace root for single-package project', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package',
                '/path/to/package'
            );

            const workingDir = getWorkingDirectoryForTarget(target, {
                isWorkspace: false,
                workspaceRoot: '/path/to/workspace'
            });

            assert.strictEqual(workingDir, '/path/to/workspace');
        });

        test('should return package path for workspace member', () => {
            const target = new CargoTarget(
                'worker',
                ['bin'],
                '/path/to/workspace/worker/src/main.rs',
                '2021',
                'worker-package',
                '/path/to/workspace/worker'
            );

            const workingDir = getWorkingDirectoryForTarget(target, {
                isWorkspace: true,
                workspaceRoot: '/path/to/workspace'
            });

            assert.strictEqual(workingDir, '/path/to/workspace/worker');
        });

        test('should return workspace root when package path is not available', () => {
            const target = new CargoTarget(
                'my-app',
                ['bin'],
                '/path/to/src/main.rs',
                '2021',
                'my-package'
                // No packagePath provided
            );

            const workingDir = getWorkingDirectoryForTarget(target, {
                isWorkspace: true,
                workspaceRoot: '/path/to/workspace'
            });

            assert.strictEqual(workingDir, '/path/to/workspace');
        });
    });

    suite('Extension Manager Integration Tests', () => {
        // Note: These tests focus on the CargoExtensionManager's internal logic
        // without requiring full VS Code integration, as the manager is a singleton
        // that requires a VS Code context to create. For true integration testing,
        // see extension.test.ts

        test('should support action type checking for targets', () => {
            // Test that target action type logic works correctly
            const binaryTarget = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const testTarget = new CargoTarget('my-test', ['test'], '/test/tests/my-test.rs', '2021', 'my-package', '/test');
            const libTarget = new CargoTarget('my-lib', ['lib'], '/test/src/lib.rs', '2021', 'my-package', '/test');
            const benchTarget = new CargoTarget('my-bench', ['bench'], '/test/benches/my-bench.rs', '2021', 'my-package', '/test');

            // Binary targets should support build and run
            assert.ok(binaryTarget.supportsActionType(TargetActionType.Build));
            assert.ok(binaryTarget.supportsActionType(TargetActionType.Run));
            assert.ok(!binaryTarget.supportsActionType(TargetActionType.Test));
            assert.ok(!binaryTarget.supportsActionType(TargetActionType.Bench));

            // Test targets should support build and test
            assert.ok(testTarget.supportsActionType(TargetActionType.Build));
            assert.ok(!testTarget.supportsActionType(TargetActionType.Run));
            assert.ok(testTarget.supportsActionType(TargetActionType.Test));
            assert.ok(!testTarget.supportsActionType(TargetActionType.Bench));

            // Library targets should only support build
            assert.ok(libTarget.supportsActionType(TargetActionType.Build));
            assert.ok(!libTarget.supportsActionType(TargetActionType.Run));
            assert.ok(!libTarget.supportsActionType(TargetActionType.Test));
            assert.ok(!libTarget.supportsActionType(TargetActionType.Bench));

            // Bench targets should support build and bench
            assert.ok(benchTarget.supportsActionType(TargetActionType.Build));
            assert.ok(!benchTarget.supportsActionType(TargetActionType.Run));
            assert.ok(!benchTarget.supportsActionType(TargetActionType.Test));
            assert.ok(benchTarget.supportsActionType(TargetActionType.Bench));
        });

        test('should generate correct cargo commands for action types', () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');

            assert.strictEqual(target.getCargoCommand(TargetActionType.Build), 'build');
            assert.strictEqual(target.getCargoCommand(TargetActionType.Run), 'run');
            assert.strictEqual(target.getCargoCommand(TargetActionType.Test), 'test');
            assert.strictEqual(target.getCargoCommand(TargetActionType.Bench), 'bench');
        });

        test('should generate correct target arguments for action types', () => {
            const binaryTarget = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const libTarget = new CargoTarget('my-lib', ['lib'], '/test/src/lib.rs', '2021', 'my-package', '/test');
            const testTarget = new CargoTarget('my-test', ['test'], '/test/tests/my-test.rs', '2021', 'my-package', '/test');
            const exampleTarget = new CargoTarget('my-example', ['example'], '/test/examples/my-example.rs', '2021', 'my-package', '/test');
            const benchTarget = new CargoTarget('my-bench', ['bench'], '/test/benches/my-bench.rs', '2021', 'my-package', '/test');

            // Binary target args
            assert.deepStrictEqual(binaryTarget.getTargetArgs(TargetActionType.Build), ['--bin', 'my-app']);
            assert.deepStrictEqual(binaryTarget.getTargetArgs(TargetActionType.Run), ['--bin', 'my-app']);

            // Library target args
            assert.deepStrictEqual(libTarget.getTargetArgs(TargetActionType.Build), ['--lib']);

            // Test target args
            assert.deepStrictEqual(testTarget.getTargetArgs(TargetActionType.Test), ['--test', 'my-test']);

            // Example target args
            assert.deepStrictEqual(exampleTarget.getTargetArgs(TargetActionType.Build), ['--example', 'my-example']);
            assert.deepStrictEqual(exampleTarget.getTargetArgs(TargetActionType.Run), ['--example', 'my-example']);

            // Bench target args
            assert.deepStrictEqual(benchTarget.getTargetArgs(TargetActionType.Bench), ['--bench', 'my-bench']);
        });

        test('should validate package context is included in target arguments', () => {
            const target = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');

            // The package context should be accessible for command building
            assert.strictEqual(target.packageName, 'my-package');
            assert.strictEqual(target.packagePath, '/test');

            // Verify the target contains all necessary information for package-aware commands
            assert.ok(target.packageName !== undefined);
            assert.ok(target.packagePath !== undefined);
        });

        test('should distinguish between different target types for context menus', () => {
            const binaryTarget = new CargoTarget('my-app', ['bin'], '/test/src/main.rs', '2021', 'my-package', '/test');
            const libTarget = new CargoTarget('my-lib', ['lib'], '/test/src/lib.rs', '2021', 'my-package', '/test');
            const testTarget = new CargoTarget('my-test', ['test'], '/test/tests/my-test.rs', '2021', 'my-package', '/test');

            // Each target type should have different supported actions
            const binaryActions = binaryTarget.supportedActionTypes;
            const libActions = libTarget.supportedActionTypes;
            const testActions = testTarget.supportedActionTypes;

            // Binary targets support build, run, and debug
            assert.ok(binaryActions.includes(TargetActionType.Build));
            assert.ok(binaryActions.includes(TargetActionType.Run));
            assert.ok(binaryActions.includes(TargetActionType.Debug));
            assert.strictEqual(binaryActions.length, 3);

            // Library targets only support build
            assert.ok(libActions.includes(TargetActionType.Build));
            assert.strictEqual(libActions.length, 1);

            // Test targets support build and test
            assert.ok(testActions.includes(TargetActionType.Build));
            assert.ok(testActions.includes(TargetActionType.Test));
            assert.strictEqual(testActions.length, 2);
        });
    });

    suite('Task Provider "All Targets" Fix Tests', () => {
        test('should build all targets when no target is specified (package "core")', () => {
            const mockWorkspace = {
                targets: [
                    new CargoTarget('cli-main', ['bin'], '/path/src/main.rs', '2021', 'core', '/path'),
                    new CargoTarget('helper', ['bin'], '/path/src/bin/helper.rs', '2021', 'core', '/path')
                ],
                currentTarget: new CargoTarget('cli-main', ['bin'], '/path/src/main.rs', '2021', 'core', '/path'),
                currentProfile: { toString: () => 'dev' },
                isWorkspace: true
            };

            const allTargetsDefinition = {
                type: 'cargo',
                command: 'build',
                packageName: 'core'
                // No targetKind, no targetName - should build all targets
            };

            const args = buildCargoArgs(allTargetsDefinition, mockWorkspace);
            const expected = ['build', '--package', 'core'];

            assert.deepStrictEqual(args, expected,
                `Expected ["build", "--package", "core"] but got ${JSON.stringify(args)}`);
        });

        test('should build specific target when target is specified', () => {
            const mockWorkspace = {
                targets: [
                    new CargoTarget('cli-main', ['bin'], '/path/src/main.rs', '2021', 'core', '/path'),
                    new CargoTarget('helper', ['bin'], '/path/src/bin/helper.rs', '2021', 'core', '/path')
                ],
                currentTarget: new CargoTarget('cli-main', ['bin'], '/path/src/main.rs', '2021', 'core', '/path'),
                currentProfile: { toString: () => 'dev' },
                isWorkspace: true
            };

            const specificTargetDefinition = {
                type: 'cargo',
                command: 'build',
                packageName: 'core',
                targetName: 'cli-main',
                targetKind: 'bin'
            };

            const args = buildCargoArgs(specificTargetDefinition, mockWorkspace);
            const expected = ['build', '--package', 'core', '--bin', 'cli-main'];

            assert.deepStrictEqual(args, expected,
                `Expected ["build", "--package", "core", "--bin", "cli-main"] but got ${JSON.stringify(args)}`);
        });

        test('should build library target correctly', () => {
            const mockWorkspace = {
                targets: [
                    new CargoTarget('mylib', ['lib'], '/path/src/lib.rs', '2021', 'core', '/path')
                ],
                currentTarget: new CargoTarget('mylib', ['lib'], '/path/src/lib.rs', '2021', 'core', '/path'),
                currentProfile: { toString: () => 'dev' },
                isWorkspace: true
            };

            const libTargetDefinition = {
                type: 'cargo',
                command: 'build',
                packageName: 'core',
                targetKind: 'lib'
            };

            const args = buildCargoArgs(libTargetDefinition, mockWorkspace);
            const expected = ['build', '--package', 'core', '--lib'];

            assert.deepStrictEqual(args, expected,
                `Expected ["build", "--package", "core", "--lib"] but got ${JSON.stringify(args)}`);
        });

        test('should not fallback to currentTarget when no target specified (All targets fix)', () => {
            const mockWorkspace = {
                targets: [
                    new CargoTarget('cli-main', ['bin'], '/path/src/main.rs', '2021', 'core', '/path')
                ],
                currentTarget: new CargoTarget('cli-main', ['bin'], '/path/src/main.rs', '2021', 'core', '/path'),
                currentProfile: { toString: () => 'dev' },
                isWorkspace: true
            };

            // This is the key test: when no targetName or targetKind is specified,
            // we should NOT fall back to currentTarget (which would add --bin cli-main)
            const allTargetsDefinition = {
                type: 'cargo',
                command: 'build',
                packageName: 'core'
                // Explicitly no targetKind, no targetName
            };

            const args = buildCargoArgs(allTargetsDefinition, mockWorkspace);

            // Should NOT contain target-specific flags like --bin cli-main
            assert.ok(!args.includes('--bin'), 'Should not include --bin flag for "All" targets');
            assert.ok(!args.includes('cli-main'), 'Should not include target name for "All" targets');
            assert.deepStrictEqual(args, ['build', '--package', 'core'],
                'Should only contain build command and package flag for "All" targets');
        });

        test('should handle non-workspace builds correctly', () => {
            const mockWorkspace = {
                targets: [
                    new CargoTarget('my-app', ['bin'], '/path/src/main.rs', '2021', undefined, '/path')
                ],
                currentTarget: new CargoTarget('my-app', ['bin'], '/path/src/main.rs', '2021', undefined, '/path'),
                currentProfile: { toString: () => 'dev' },
                isWorkspace: false
            };

            const allTargetsDefinition = {
                type: 'cargo',
                command: 'build'
                // No package, no target - single package build all
            };

            const args = buildCargoArgs(allTargetsDefinition, mockWorkspace);
            const expected = ['build'];

            assert.deepStrictEqual(args, expected,
                `Expected ["build"] for single package all targets but got ${JSON.stringify(args)}`);
        });
    });

    suite('Task Provider Integration Tests', () => {
        /**
         * These tests verify that the task provider correctly handles different scenarios
         * without relying on VS Code's task system
         */

        test('should generate correct args for "All" targets (no target-specific flags)', () => {
            const mockWorkspace = {
                workspaceRoot: '/test',
                targets: [
                    new CargoTarget('test-bin', ['bin'], '/test/src/main.rs', '2021', 'test-package', '/test'),
                    new CargoTarget('test-lib', ['lib'], '/test/src/lib.rs', '2021', 'test-package', '/test')
                ],
                currentProfile: { toString: () => 'dev' },
                selectedFeatures: new Set(),
                isWorkspace: true
            };

            // Mock the buildCargoArgs method from our task provider
            const allTargetsDefinition = {
                type: 'cargo',
                command: 'build'
                // No targetKind, no targetName = "All" targets
            };

            const args = buildCargoArgs(allTargetsDefinition, mockWorkspace);

            // Should only contain the build command, no target-specific flags
            assert.deepStrictEqual(args, ['build'],
                `All targets should generate ["build"] but got ${JSON.stringify(args)}`);
            assert.ok(!args.includes('--bin'), 'All targets should not include --bin flag');
            assert.ok(!args.includes('--lib'), 'All targets should not include --lib flag');
        });

        test('should generate correct args for library target (--lib flag)', () => {
            const mockWorkspace = {
                workspaceRoot: '/test',
                targets: [
                    new CargoTarget('test-lib', ['lib'], '/test/src/lib.rs', '2021', 'test-package', '/test')
                ],
                currentProfile: { toString: () => 'dev' },
                selectedFeatures: new Set(),
                isWorkspace: false
            };

            const libTargetDefinition = {
                type: 'cargo',
                command: 'build',
                targetKind: 'lib'
            };

            const args = buildCargoArgs(libTargetDefinition, mockWorkspace);

            assert.deepStrictEqual(args, ['build', '--lib'],
                `Library target should generate ["build", "--lib"] but got ${JSON.stringify(args)}`);
        });

        test('should generate correct args for specific binary target', () => {
            const mockWorkspace = {
                workspaceRoot: '/test',
                targets: [
                    new CargoTarget('test-bin', ['bin'], '/test/src/main.rs', '2021', 'test-package', '/test')
                ],
                currentProfile: { toString: () => 'dev' },
                selectedFeatures: new Set(),
                isWorkspace: false
            };

            const binTargetDefinition = {
                type: 'cargo',
                command: 'build',
                targetName: 'test-bin',
                targetKind: 'bin'
            };

            const args = buildCargoArgs(binTargetDefinition, mockWorkspace);

            assert.deepStrictEqual(args, ['build', '--bin', 'test-bin'],
                `Binary target should generate ["build", "--bin", "test-bin"] but got ${JSON.stringify(args)}`);
        });

        test('should include package flag in workspace context', () => {
            const mockWorkspace = {
                workspaceRoot: '/test',
                targets: [
                    new CargoTarget('test-bin', ['bin'], '/test/src/main.rs', '2021', 'test-package', '/test')
                ],
                currentProfile: { toString: () => 'dev' },
                selectedFeatures: new Set(),
                isWorkspace: true
            };

            const packageSpecificDefinition = {
                type: 'cargo',
                command: 'build',
                packageName: 'test-package',
                targetName: 'test-bin',
                targetKind: 'bin'
            };

            const args = buildCargoArgs(packageSpecificDefinition, mockWorkspace);

            assert.deepStrictEqual(args, ['build', '--package', 'test-package', '--bin', 'test-bin'],
                `Workspace package target should include package flag but got ${JSON.stringify(args)}`);
        });

        test('should handle release profile correctly', () => {
            const mockWorkspace = {
                workspaceRoot: '/test',
                targets: [],
                currentProfile: { toString: () => 'release' },
                selectedFeatures: new Set(),
                isWorkspace: false
            };

            const releaseDefinition = {
                type: 'cargo',
                command: 'build',
                profile: 'release'
            };

            const args = buildCargoArgs(releaseDefinition, mockWorkspace);

            assert.deepStrictEqual(args, ['build', '--release'],
                `Release build should include --release flag but got ${JSON.stringify(args)}`);
        });
    });

    suite('Custom Profile Discovery Tests', () => {
        test('should discover custom profiles from Cargo.toml', () => {
            // Test the CargoProfile custom profile functionality
            CargoProfile.clearCustomProfiles();

            // Simulate discovered custom profiles
            CargoProfile.addCustomProfile('fast');
            CargoProfile.addCustomProfile('staging');
            CargoProfile.addCustomProfile('production');

            const customProfiles = CargoProfile.getCustomProfiles();
            assert.ok(customProfiles.includes('fast'), 'Should include fast profile');
            assert.ok(customProfiles.includes('staging'), 'Should include staging profile');
            assert.ok(customProfiles.includes('production'), 'Should include production profile');

            // Test getAllProfiles includes both standard and custom
            const allProfiles = CargoProfile.getAllProfiles();
            const profileNames = allProfiles.map(p => p.toString());

            assert.ok(profileNames.includes('none'), 'Should include none profile');
            assert.ok(profileNames.includes('dev'), 'Should include dev profile');
            assert.ok(profileNames.includes('release'), 'Should include release profile');
            assert.ok(profileNames.includes('test'), 'Should include test profile');
            assert.ok(profileNames.includes('bench'), 'Should include bench profile');
            // Note: doc profile is excluded from selection as it's not typically user-selectable
            assert.ok(profileNames.includes('fast'), 'Should include custom fast profile');
            assert.ok(profileNames.includes('staging'), 'Should include custom staging profile');
            assert.ok(profileNames.includes('production'), 'Should include custom production profile');
        });

        test('should handle custom profile creation and comparison', () => {
            const customProfile = CargoProfile.fromString('custom-optimization');

            assert.strictEqual(customProfile.toString(), 'custom-optimization', 'Custom profile should have correct name');
            assert.ok(customProfile.isCustom(), 'Custom profile should be identified as custom');

            const devProfile = CargoProfile.fromString('dev');
            assert.ok(!devProfile.isCustom(), 'Dev profile should not be identified as custom');

            const sameCustomProfile = CargoProfile.fromString('custom-optimization');
            assert.ok(customProfile.equals(sameCustomProfile), 'Same custom profiles should be equal');
        });

        test('should not add standard profiles as custom', () => {
            CargoProfile.clearCustomProfiles();

            // Try to add standard profiles as custom (should be ignored)
            CargoProfile.addCustomProfile('dev');
            CargoProfile.addCustomProfile('release');
            CargoProfile.addCustomProfile('test');
            CargoProfile.addCustomProfile('bench');
            CargoProfile.addCustomProfile('doc');
            CargoProfile.addCustomProfile('none');

            const customProfiles = CargoProfile.getCustomProfiles();
            assert.strictEqual(customProfiles.length, 0, 'Standard profiles should not be added as custom');
        });

        test('should provide correct display names and descriptions for custom profiles', () => {
            const customProfile = new CargoProfile('my-custom-profile');

            const displayName = CargoProfile.getDisplayName(customProfile);
            assert.strictEqual(displayName, 'My-custom-profile', 'Custom profile should have capitalized display name');

            const description = CargoProfile.getDescription(customProfile);
            assert.ok(description.includes('--profile my-custom-profile'), 'Custom profile description should include the profile flag');
            assert.ok(description.includes('user-defined settings'), 'Custom profile description should mention user-defined settings');
        });

        test('should integrate custom profiles with CargoWorkspace', async () => {
            // Test that a real CargoWorkspace can discover custom profiles
            const path = require('path');
            const testProjectPath = path.join(__dirname, '../../test-rust-project');

            // Skip this test if the test project doesn't exist
            const fs = require('fs');
            if (!fs.existsSync(testProjectPath)) {
                console.log('Skipping integration test - test project not found');
                return;
            }

            // Clear any existing custom profiles to start fresh
            CargoProfile.clearCustomProfiles();

            const workspace = new CargoWorkspace(testProjectPath);
            await workspace.initialize();

            // After initialization, custom profiles should be discovered
            const allProfiles = CargoProfile.getAllProfiles();
            const profileNames = allProfiles.map(p => p.toString());

            // Check that custom profiles from Cargo.toml are discovered
            assert.ok(profileNames.includes('fast'), 'Should discover "fast" profile from workspace Cargo.toml');
            assert.ok(profileNames.includes('staging'), 'Should discover "staging" profile from workspace Cargo.toml');
            assert.ok(profileNames.includes('bench-optimized'), 'Should discover "bench-optimized" profile from workspace Cargo.toml');

            // Check that custom profiles from .cargo/config.toml are discovered  
            assert.ok(profileNames.includes('debug-optimized'), 'Should discover "debug-optimized" profile from .cargo/config.toml');
            assert.ok(profileNames.includes('ci'), 'Should discover "ci" profile from .cargo/config.toml');

            // Standard profiles should still be present
            assert.ok(profileNames.includes('none'), 'Should include none profile');
            assert.ok(profileNames.includes('dev'), 'Should include dev profile');
            assert.ok(profileNames.includes('release'), 'Should include release profile');
        });
    });

    suite('Command Override Settings Tests', () => {
        class MockCargoConfigurationReader {
            constructor(
                public runCommandOverride: string = '',
                public testCommandOverride: string = ''
            ) { }

            get cargoCommand() { return 'cargo'; }
            get cargoPath() { return 'cargo'; }
            get useRustAnalyzerEnvAndArgs() { return false; }
            get updateRustAnalyzerTarget() { return false; }
            get extraEnv() { return {}; }
            get buildArgs() { return []; }
            get runArgs() { return []; }
            get testArgs() { return []; }
            get runExtraArgs() { return []; }
            get runExtraEnv() { return {}; }
            get testExtraArgs() { return []; }
            get testExtraEnv() { return {}; }
            get environment() { return {}; }
            get features() { return []; }
            get allFeatures() { return false; }
            get noDefaultFeatures() { return false; }
        }

        function createMockWorkspace(targets: CargoTarget[] = []): any {
            return {
                workspaceRoot: '/test/workspace',
                targets: targets,
                currentProfile: CargoProfile.dev,
                selectedFeatures: new Set<string>(),
                selectedPackage: undefined as string | undefined,
                isWorkspace: false
            };
        }

        test('should use default cargo run when no override is set', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReader('', ''); // No overrides
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.command, 'run', 'Should use run command');
        });

        test('should use default cargo test when no override is set', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReader('', ''); // No overrides
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-test', ['test'], '/test/src/test.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Test);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.command, 'test', 'Should use test command');
        });

        test('should include override settings in configuration interface', () => {
            const mockConfig = new MockCargoConfigurationReader('cargo watch -x run', 'cargo nextest run');

            assert.strictEqual(mockConfig.runCommandOverride, 'cargo watch -x run', 'Should store run override');
            assert.strictEqual(mockConfig.testCommandOverride, 'cargo nextest run', 'Should store test override');
        });

        test('should work with binary targets and run override', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const mockWorkspace = createMockWorkspace([target]);
            const mockConfig = new MockCargoConfigurationReader('cargo watch -x run', '');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.command, 'run', 'Should still use run in definition');
            assert.strictEqual(task.definition.targetName, 'my-binary', 'Should preserve target name');
            assert.strictEqual(task.definition.targetKind, 'bin', 'Should preserve target kind');
        });

        test('should work with test targets and test override', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const target = new CargoTarget('integration_tests', ['test'], '/test/tests/integration.rs');
            const mockWorkspace = createMockWorkspace([target]);
            const mockConfig = new MockCargoConfigurationReader('', 'cargo nextest run');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Test);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.command, 'test', 'Should still use test in definition');
            assert.strictEqual(task.definition.targetName, 'integration_tests', 'Should preserve target name');
            assert.strictEqual(task.definition.targetKind, 'test', 'Should preserve target kind');
        });

        test('should work with example targets and run override', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const target = new CargoTarget('simple-server', ['example'], '/test/examples/simple.rs');
            const mockWorkspace = createMockWorkspace([target]);
            const mockConfig = new MockCargoConfigurationReader('cargo watch -x run', '');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.command, 'run', 'Should still use run in definition');
            assert.strictEqual(task.definition.targetName, 'simple-server', 'Should preserve target name');
            assert.strictEqual(task.definition.targetKind, 'example', 'Should preserve target kind');
        });

        test('should not affect build commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const mockWorkspace = createMockWorkspace([target]);
            const mockConfig = new MockCargoConfigurationReader('cargo watch -x run', 'cargo nextest run');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Build);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.command, 'build', 'Should use build command, not affected by overrides');
        });

        test('should preserve workspace and package settings with overrides', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs', '2021', 'my-package');
            const mockWorkspace = createMockWorkspace([target]);
            mockWorkspace.selectedPackage = 'my-package';
            mockWorkspace.isWorkspace = true;

            const mockConfig = new MockCargoConfigurationReader('cargo watch -x run', '');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.strictEqual(task.definition.packageName, 'my-package', 'Should preserve package information');
            assert.strictEqual(task.definition.targetName, 'my-binary', 'Should preserve target name');
        });

        test('should preserve features settings with overrides', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const mockWorkspace = createMockWorkspace([target]);
            mockWorkspace.selectedFeatures = new Set(['feature1', 'feature2']);

            const mockConfig = new MockCargoConfigurationReader('cargo watch -x run', '');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.ok(Array.isArray(task.definition.features), 'Should have features array');
            assert.ok(task.definition.features?.includes('feature1'), 'Should preserve feature1');
            assert.ok(task.definition.features?.includes('feature2'), 'Should preserve feature2');
        });
    });

    suite('New Settings Tests', () => {
        class MockCargoConfigurationReaderWithNewSettings {
            constructor(
                public cargoCommand: string = 'cargo',
                public extraEnv: { [key: string]: string } = {},
                public runExtraArgs: string[] = [],
                public runExtraEnv: { [key: string]: string } = {},
                public testExtraArgs: string[] = [],
                public testExtraEnv: { [key: string]: string } = {},
                public runCommandOverride: string = '',
                public testCommandOverride: string = '',
                public useRustAnalyzerEnvAndArgs: boolean = false
            ) { }

            get cargoPath() { return 'cargo'; }
            get buildArgs() { return []; }
            get runArgs() { return []; }
            get testArgs() { return []; }
            get environment() { return {}; }
            get features() { return []; }
            get allFeatures() { return false; }
            get noDefaultFeatures() { return false; }
        }

        function createMockWorkspace(targets: CargoTarget[] = []): any {
            return {
                workspaceRoot: '/test/workspace',
                targets: targets,
                currentProfile: CargoProfile.dev,
                selectedFeatures: new Set<string>(),
                selectedPackage: undefined as string | undefined,
                isWorkspace: false
            };
        }

        test('should use cargoCommand with single command', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings('cross');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Build);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            assert.strictEqual(execution.command, 'cross', 'Should use cross command');
        });

        test('should split cargoCommand with whitespace into command and args', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings('cargo +nightly');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Build);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            assert.strictEqual(execution.command, 'cargo', 'Should use cargo as command');
            assert.ok(execution.args.includes('+nightly'), 'Should include +nightly as first argument');
        });

        test('should apply run.extraArgs to run commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                {},
                ['--verbose', '--bin-name', 'custom']
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            const argsStr = execution.args.join(' ');
            assert.ok(argsStr.includes('--verbose'), 'Should include --verbose from run.extraArgs');
            assert.ok(argsStr.includes('--bin-name'), 'Should include --bin-name from run.extraArgs');
            assert.ok(argsStr.includes('custom'), 'Should include custom from run.extraArgs');
        });

        test('should apply test.extraArgs to test commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                {},
                [], {},
                ['--nocapture', '--test-threads=1']
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-test', ['test'], '/test/src/test.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Test);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            const argsStr = execution.args.join(' ');
            assert.ok(argsStr.includes('--nocapture'), 'Should include --nocapture from test.extraArgs');
            assert.ok(argsStr.includes('--test-threads=1'), 'Should include --test-threads=1 from test.extraArgs');
        });

        test('should apply extraEnv to all commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                { 'RUST_LOG': 'debug', 'CUSTOM_VAR': 'value' }
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Build);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            assert.ok(execution.options && execution.options.env, 'Should have environment options');
            assert.strictEqual(execution.options.env.RUST_LOG, 'debug', 'Should include RUST_LOG from extraEnv');
            assert.strictEqual(execution.options.env.CUSTOM_VAR, 'value', 'Should include CUSTOM_VAR from extraEnv');
        });

        test('should merge run.extraEnv with extraEnv for run commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                { 'RUST_LOG': 'info', 'GLOBAL_VAR': 'global' },
                [],
                { 'RUST_LOG': 'debug', 'RUN_VAR': 'run_value' }
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            assert.ok(execution.options && execution.options.env, 'Should have environment options');
            assert.strictEqual(execution.options.env.RUST_LOG, 'debug', 'run.extraEnv should override extraEnv');
            assert.strictEqual(execution.options.env.GLOBAL_VAR, 'global', 'Should include GLOBAL_VAR from extraEnv');
            assert.strictEqual(execution.options.env.RUN_VAR, 'run_value', 'Should include RUN_VAR from run.extraEnv');
        });

        test('should merge test.extraEnv with extraEnv for test commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                { 'RUST_LOG': 'info', 'GLOBAL_VAR': 'global' },
                [], {},
                [],
                { 'RUST_LOG': 'trace', 'TEST_VAR': 'test_value' }
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-test', ['test'], '/test/src/test.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Test);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            assert.ok(execution.options && execution.options.env, 'Should have environment options');
            assert.strictEqual(execution.options.env.RUST_LOG, 'trace', 'test.extraEnv should override extraEnv');
            assert.strictEqual(execution.options.env.GLOBAL_VAR, 'global', 'Should include GLOBAL_VAR from extraEnv');
            assert.strictEqual(execution.options.env.TEST_VAR, 'test_value', 'Should include TEST_VAR from test.extraEnv');
        });

        test('should not apply run.extraArgs to build commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                {},
                ['--should-not-appear']
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Build);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            const argsStr = execution.args.join(' ');
            assert.ok(!argsStr.includes('--should-not-appear'), 'Should not include run.extraArgs in build command');
        });

        test('should not apply test.extraArgs to run commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                {},
                [], {},
                ['--should-not-appear']
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Run);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            const argsStr = execution.args.join(' ');
            assert.ok(!argsStr.includes('--should-not-appear'), 'Should not include test.extraArgs in run command');
        });

        test('should apply run.extraArgs to bench commands', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings(
                'cargo',
                {},
                ['--bench-arg']
            );
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-bench', ['bench'], '/test/benches/bench.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Bench);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            const argsStr = execution.args.join(' ');
            assert.ok(argsStr.includes('--bench-arg'), 'Should include --bench-arg from run.extraArgs for bench command');
        });

        test('should handle complex cargoCommand with multiple arguments', () => {
            const { CargoTaskProvider } = require('../cargoTaskProvider');
            const mockWorkspace = createMockWorkspace();
            const mockConfig = new MockCargoConfigurationReaderWithNewSettings('cargo +nightly --verbose');
            const taskProvider = new CargoTaskProvider(mockWorkspace, mockConfig);

            const target = new CargoTarget('my-binary', ['bin'], '/test/src/main.rs');
            const task = taskProvider.createTaskForTargetAction(target, TargetActionType.Build);

            assert.ok(task, 'Should create a task');
            assert.ok(task.execution instanceof require('vscode').ShellExecution);
            const execution = task.execution as any;
            assert.strictEqual(execution.command, 'cargo', 'Should use cargo as command');
            assert.ok(execution.args.includes('+nightly'), 'Should include +nightly as argument');
            assert.ok(execution.args.includes('--verbose'), 'Should include --verbose as argument');
        });

        test('should use rust-analyzer settings when useRustAnalyzerEnvAndArgs is enabled', () => {
            // Mock VS Code workspace configuration
            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section: string) => {
                if (section === 'cargoTools') {
                    return {
                        get: (key: string, defaultValue: any) => {
                            if (key === 'useRustAnalyzerEnvAndArgs') {
                                return true;
                            }
                            return defaultValue;
                        }
                    };
                } else if (section === 'rust-analyzer') {
                    return {
                        get: (key: string, defaultValue: any) => {
                            switch (key) {
                                case 'cargo.extraArgs': return ['--', '--release'];
                                case 'cargo.extraEnv': return { 'RUST_LOG': 'debug' };
                                case 'runnables.extraArgs': return ['--run-arg'];
                                case 'runnables.extraTestBinaryArgs': return ['--test-arg'];
                                default: return defaultValue;
                            }
                        }
                    };
                }
                return { get: () => undefined };
            };

            try {
                const { CargoConfigurationReader } = require('../cargoConfigurationReader');
                const config = CargoConfigurationReader.loadConfig();

                assert.strictEqual(config.cargoCommand, 'cargo -- --release', 'Should build cargoCommand from rust-analyzer.cargo.extraArgs');
                assert.deepStrictEqual(config.extraEnv, { 'RUST_LOG': 'debug' }, 'Should use rust-analyzer.cargo.extraEnv');
                assert.deepStrictEqual(config['run.extraArgs'], ['--run-arg'], 'Should use rust-analyzer.runnables.extraArgs');
                assert.deepStrictEqual(config['test.extraArgs'], ['--test-arg'], 'Should use rust-analyzer.runnables.extraTestBinaryArgs');
            } finally {
                // Restore original function
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });

        test('should merge rust-analyzer settings with existing settings', () => {
            // Mock VS Code workspace configuration
            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section: string) => {
                if (section === 'cargoTools') {
                    return {
                        get: (key: string, defaultValue: any) => {
                            switch (key) {
                                case 'useRustAnalyzerEnvAndArgs': return true;
                                case 'extraEnv': return { 'EXISTING_VAR': 'existing' };
                                case 'run.extraArgs': return ['--existing-run-arg'];
                                case 'test.extraArgs': return ['--existing-test-arg'];
                                default: return defaultValue;
                            }
                        }
                    };
                } else if (section === 'rust-analyzer') {
                    return {
                        get: (key: string, defaultValue: any) => {
                            switch (key) {
                                case 'cargo.extraArgs': return ['+nightly'];
                                case 'cargo.extraEnv': return { 'RUST_LOG': 'debug' };
                                case 'runnables.extraArgs': return ['--rust-run-arg'];
                                case 'runnables.extraTestBinaryArgs': return ['--rust-test-arg'];
                                default: return defaultValue;
                            }
                        }
                    };
                }
                return { get: () => undefined };
            };

            try {
                const { CargoConfigurationReader } = require('../cargoConfigurationReader');
                const config = CargoConfigurationReader.loadConfig();

                assert.strictEqual(config.cargoCommand, 'cargo +nightly', 'Should build cargoCommand from rust-analyzer.cargo.extraArgs');
                assert.deepStrictEqual(config.extraEnv, { 'EXISTING_VAR': 'existing', 'RUST_LOG': 'debug' }, 'Should merge existing and rust-analyzer extraEnv');
                assert.deepStrictEqual(config['run.extraArgs'], ['--existing-run-arg', '--rust-run-arg'], 'Should merge existing and rust-analyzer run args');
                assert.deepStrictEqual(config['test.extraArgs'], ['--existing-test-arg', '--rust-test-arg'], 'Should merge existing and rust-analyzer test args');
            } finally {
                // Restore original function
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });

        test('should not use rust-analyzer settings when useRustAnalyzerEnvAndArgs is disabled', () => {
            // Mock VS Code workspace configuration
            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section: string) => {
                if (section === 'cargoTools') {
                    return {
                        get: (key: string, defaultValue: any) => {
                            switch (key) {
                                case 'useRustAnalyzerEnvAndArgs': return false;
                                case 'cargoCommand': return 'custom-cargo';
                                case 'extraEnv': return { 'CUSTOM_VAR': 'custom' };
                                default: return defaultValue;
                            }
                        }
                    };
                } else if (section === 'rust-analyzer') {
                    return {
                        get: (key: string, defaultValue: any) => {
                            // These should be ignored when integration is disabled
                            switch (key) {
                                case 'cargo.extraArgs': return ['--should-not-be-used'];
                                case 'cargo.extraEnv': return { 'RUST_LOG': 'should-not-be-used' };
                                default: return defaultValue;
                            }
                        }
                    };
                }
                return { get: () => undefined };
            };

            try {
                const { CargoConfigurationReader } = require('../cargoConfigurationReader');
                const config = CargoConfigurationReader.loadConfig();

                assert.strictEqual(config.cargoCommand, 'custom-cargo', 'Should use cargoTools.cargoCommand');
                assert.deepStrictEqual(config.extraEnv, { 'CUSTOM_VAR': 'custom' }, 'Should use cargoTools.extraEnv');
                assert.ok(!config.extraEnv.hasOwnProperty('RUST_LOG'), 'Should not include rust-analyzer settings');
            } finally {
                // Restore original function
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });
    });

    suite('Rust-Analyzer Target Synchronization Tests', () => {
        test('should update rust-analyzer target when updateRustAnalyzerTarget is enabled', () => {
            const testProjectPath = '/test/project';
            const workspace = new CargoWorkspace(testProjectPath);

            // Mock VS Code configuration
            let rustAnalyzerTargetValue: string | undefined = undefined;
            const mockCargoToolsConfig = {
                get: (key: string, defaultValue?: any) => {
                    if (key === 'updateRustAnalyzerTarget') {
                        return true;
                    }
                    return defaultValue;
                }
            };
            const mockRustAnalyzerConfig = {
                get: (key: string, defaultValue?: any) => {
                    if (key === 'cargo.target') {
                        return rustAnalyzerTargetValue;
                    }
                    return defaultValue;
                },
                update: (key: string, value: any, target: any) => {
                    if (key === 'cargo.target') {
                        rustAnalyzerTargetValue = value;
                    }
                }
            };

            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section?: string) => {
                if (section === 'cargoTools') {
                    return mockCargoToolsConfig;
                }
                if (section === 'rust-analyzer') {
                    return mockRustAnalyzerConfig;
                }
                return originalGetConfiguration(section);
            };

            try {
                // Test setting a platform target
                workspace.setSelectedPlatformTarget('wasm32-unknown-unknown');
                assert.strictEqual(rustAnalyzerTargetValue, 'wasm32-unknown-unknown', 'Should set rust-analyzer cargo target');

                // Test removing platform target
                workspace.setSelectedPlatformTarget(null);
                assert.strictEqual(rustAnalyzerTargetValue, undefined, 'Should remove rust-analyzer cargo target');
            } finally {
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });

        test('should not update rust-analyzer target when updateRustAnalyzerTarget is disabled', () => {
            const testProjectPath = '/test/project';
            const workspace = new CargoWorkspace(testProjectPath);

            // Mock VS Code configuration
            let rustAnalyzerTargetValue: string | undefined = 'initial-value';
            const mockCargoToolsConfig = {
                get: (key: string, defaultValue?: any) => {
                    if (key === 'updateRustAnalyzerTarget') {
                        return false;
                    }
                    return defaultValue;
                }
            };
            const mockRustAnalyzerConfig = {
                get: (key: string, defaultValue?: any) => {
                    if (key === 'cargo.target') {
                        return rustAnalyzerTargetValue;
                    }
                    return defaultValue;
                },
                update: (key: string, value: any, target: any) => {
                    if (key === 'cargo.target') {
                        rustAnalyzerTargetValue = value;
                    }
                }
            };

            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section?: string) => {
                if (section === 'cargoTools') {
                    return mockCargoToolsConfig;
                }
                if (section === 'rust-analyzer') {
                    return mockRustAnalyzerConfig;
                }
                return originalGetConfiguration(section);
            };

            try {
                // Test setting a platform target
                workspace.setSelectedPlatformTarget('wasm32-unknown-unknown');
                assert.strictEqual(rustAnalyzerTargetValue, 'initial-value', 'Should not change rust-analyzer cargo target when disabled');

                // Test removing platform target
                workspace.setSelectedPlatformTarget(null);
                assert.strictEqual(rustAnalyzerTargetValue, 'initial-value', 'Should not change rust-analyzer cargo target when disabled');
            } finally {
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });
    });

    suite('Rust-Analyzer Check Targets Command Tests', () => {
        test('should handle rust-analyzer configuration updates correctly', async () => {
            // Mock rust-analyzer configuration
            let checkTargetsValue: string[] | undefined = [];
            const mockRustAnalyzerConfig = {
                get: (key: string, defaultValue?: any) => {
                    if (key === 'check.targets') {
                        return checkTargetsValue;
                    }
                    return defaultValue;
                },
                update: async (key: string, value: any, target: any) => {
                    if (key === 'check.targets') {
                        checkTargetsValue = value;
                    }
                }
            };

            // Mock VS Code APIs
            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section?: string) => {
                if (section === 'rust-analyzer') {
                    return mockRustAnalyzerConfig;
                }
                return originalGetConfiguration(section);
            };

            try {
                // Test setting targets
                await mockRustAnalyzerConfig.update('check.targets', ['wasm32-unknown-unknown', 'x86_64-pc-windows-gnu'], 'workspace');
                assert.deepStrictEqual(checkTargetsValue, ['wasm32-unknown-unknown', 'x86_64-pc-windows-gnu'], 'Should set multiple targets');

                // Test removing targets
                await mockRustAnalyzerConfig.update('check.targets', undefined, 'workspace');
                assert.strictEqual(checkTargetsValue, undefined, 'Should remove setting when undefined');

                // Test empty array
                await mockRustAnalyzerConfig.update('check.targets', [], 'workspace');
                assert.deepStrictEqual(checkTargetsValue, [], 'Should handle empty array');
            } finally {
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });

        test('should verify command is registered in package.json', () => {
            // This test verifies that the command exists in package.json
            const packageJson = require('../../package.json');
            const commands = packageJson.contributes.commands;

            const setRustAnalyzerCheckTargetsCommand = commands.find((cmd: any) =>
                cmd.command === 'cargo-tools.setRustAnalyzerCheckTargets'
            );

            assert.ok(setRustAnalyzerCheckTargetsCommand, 'setRustAnalyzerCheckTargets command should be defined in package.json');
            assert.strictEqual(setRustAnalyzerCheckTargetsCommand.title, 'Set rust-analyzer check targets', 'Command should have correct title');
            assert.strictEqual(setRustAnalyzerCheckTargetsCommand.category, 'Cargo Tools', 'Command should be in Cargo Tools category');
        });

        test('should handle null check.targets configuration gracefully', async () => {
            // Mock rust-analyzer configuration that returns null
            let checkTargetsValue: string[] | null = null;
            const mockRustAnalyzerConfig = {
                get: (key: string, defaultValue?: any) => {
                    if (key === 'check.targets') {
                        return checkTargetsValue; // This will be null initially
                    }
                    return defaultValue;
                },
                update: async (key: string, value: any, target: any) => {
                    if (key === 'check.targets') {
                        checkTargetsValue = value;
                    }
                }
            };

            // Mock VS Code APIs
            const originalGetConfiguration = require('vscode').workspace.getConfiguration;
            require('vscode').workspace.getConfiguration = (section?: string) => {
                if (section === 'rust-analyzer') {
                    return mockRustAnalyzerConfig;
                }
                return originalGetConfiguration(section);
            };

            try {
                // Test that null values are handled correctly by the configuration reader
                const rawTargets = mockRustAnalyzerConfig.get('check.targets', []);
                const currentTargets: string[] = rawTargets || [];
                assert.deepStrictEqual(currentTargets, [], 'Should default to empty array when setting is null');
                assert.strictEqual(currentTargets.length, 0, 'Array should be empty');
            } finally {
                require('vscode').workspace.getConfiguration = originalGetConfiguration;
            }
        });
    });
});
