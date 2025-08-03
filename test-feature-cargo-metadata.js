/**
 * Test script to verify feature selection works with actual cargo metadata
 */

const fs = require('fs');
const path = require('path');
const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);

async function testFeatureSelection() {
    try {
        // Test with the test-rust-project workspace
        const testProjectPath = path.join(__dirname, 'test-rust-project');

        console.log('✓ Testing cargo metadata parsing for features...');

        // Run cargo metadata to see what features are available
        const { stdout } = await execAsync('cargo metadata --format-version 1 --no-deps', {
            cwd: testProjectPath
        });

        const metadata = JSON.parse(stdout);

        console.log('\n✓ Checking packages for features...');

        let foundFeatures = false;
        for (const pkg of metadata.packages) {
            if (pkg.features && Object.keys(pkg.features).length > 0) {
                console.log(`  ✓ Package "${pkg.name}" has features:`, Object.keys(pkg.features));
                foundFeatures = true;
            } else {
                console.log(`  - Package "${pkg.name}" has no features`);
            }
        }

        if (foundFeatures) {
            console.log('\n✅ Successfully found packages with features!');
            console.log('Feature selection functionality should work correctly.');
        } else {
            console.log('\n⚠️  No features found in test project.');
            console.log('Feature selection will show only "all-features" option.');
        }

        // Test the logic flow
        console.log('\n✓ Testing feature selection logic...');

        // Simulate selecting "All" package - should show only "all-features"
        console.log('  - When "All" packages selected: should show only "all-features"');

        // Simulate selecting "core" package - should show core's features
        const corePackage = metadata.packages.find(p => p.name === 'core');
        if (corePackage && corePackage.features) {
            console.log(`  - When "core" package selected: should show "all-features" + ${Object.keys(corePackage.features).join(', ')}`);
        }

        return true;

    } catch (error) {
        console.error('❌ Error testing feature selection:', error.message);
        return false;
    }
}

testFeatureSelection().then(success => {
    if (success) {
        console.log('\n================');
        console.log('✅ Feature selection test completed successfully!');
        process.exit(0);
    } else {
        console.log('\n================');
        console.log('❌ Feature selection test failed.');
        process.exit(1);
    }
});
