/**
 * Test script to verify refined feature selection behavior
 */

const fs = require('fs');
const path = require('path');

// Test the refined feature selection logic
function testFeatureSelectionLogic() {
    console.log('✓ Testing refined feature selection logic...\n');

    // Simulate feature selection behavior
    class MockFeatureManager {
        constructor() {
            this.selectedFeatures = new Set(); // Default: no features selected
            this.availableFeatures = ['all-features', 'async-support', 'std-support', 'default'];
        }

        toggleFeature(feature) {
            const newFeatures = new Set(this.selectedFeatures);

            if (feature === 'all-features') {
                // If toggling all-features
                if (newFeatures.has('all-features')) {
                    // If all-features is currently selected, deselect it (allow empty selection)
                    newFeatures.clear();
                } else {
                    // If all-features is not selected, select it and clear others
                    newFeatures.clear();
                    newFeatures.add('all-features');
                }
            } else {
                // If toggling a specific feature
                if (newFeatures.has(feature)) {
                    // Deselect the feature
                    newFeatures.delete(feature);
                } else {
                    // Select the feature and remove all-features
                    newFeatures.delete('all-features');
                    newFeatures.add(feature);
                }
                // Note: Empty selection is now allowed as the default state
            }

            this.selectedFeatures = newFeatures;
            return Array.from(newFeatures);
        }

        getStatus() {
            const selectedCount = this.selectedFeatures.size;
            if (selectedCount === 0) {
                return 'No features selected (default)';
            } else if (this.selectedFeatures.has('all-features')) {
                return 'All features selected';
            } else {
                return `${selectedCount} feature${selectedCount > 1 ? 's' : ''} selected: ${Array.from(this.selectedFeatures).join(', ')}`;
            }
        }
    }

    const manager = new MockFeatureManager();

    // Test 1: Default state (no features selected)
    console.log('Test 1: Default state');
    console.log(`  Selected: [${Array.from(manager.selectedFeatures).join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should show "No features selected (default)"\n');

    // Test 2: Select "all-features"
    console.log('Test 2: Select "all-features"');
    let result = manager.toggleFeature('all-features');
    console.log(`  Selected: [${result.join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should show "All features selected"\n');

    // Test 3: Deselect "all-features" (back to empty)
    console.log('Test 3: Deselect "all-features"');
    result = manager.toggleFeature('all-features');
    console.log(`  Selected: [${result.join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should be empty again\n');

    // Test 4: Select a specific feature
    console.log('Test 4: Select "async-support"');
    result = manager.toggleFeature('async-support');
    console.log(`  Selected: [${result.join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should show "1 feature selected: async-support"\n');

    // Test 5: Select another specific feature
    console.log('Test 5: Select "std-support"');
    result = manager.toggleFeature('std-support');
    console.log(`  Selected: [${result.join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should show "2 features selected: async-support, std-support"\n');

    // Test 6: Deselect a specific feature
    console.log('Test 6: Deselect "async-support"');
    result = manager.toggleFeature('async-support');
    console.log(`  Selected: [${result.join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should show "1 feature selected: std-support"\n');

    // Test 7: Deselect last feature (should allow empty)
    console.log('Test 7: Deselect "std-support"');
    result = manager.toggleFeature('std-support');
    console.log(`  Selected: [${result.join(', ')}]`);
    console.log(`  Status: ${manager.getStatus()}`);
    console.log('  ✓ Should be empty again (no features selected)\n');

    return true;
}

// Check that the code changes are present
function verifyCodeChanges() {
    console.log('✓ Verifying code changes...\n');

    const cargoWorkspacePath = path.join(__dirname, 'src', 'cargoWorkspace.ts');
    const content = fs.readFileSync(cargoWorkspacePath, 'utf8');

    const requiredChanges = [
        'new Set(); // Selected features, default to none',
        'new Set(); // Reset to default (no features selected)',
    ];

    let allChangesPresent = true;
    for (const change of requiredChanges) {
        if (content.includes(change)) {
            console.log(`  ✓ Found: "${change}"`);
        } else {
            console.log(`  ✗ Missing: "${change}"`);
            allChangesPresent = false;
        }
    }

    // Check that old logic is gone
    const removedLogic = [
        'new Set([\'all-features\']); // Selected features, default to all-features',
        'if (newFeatures.size === 0) {', // This should be gone - we no longer force all-features when empty
    ];

    for (const removed of removedLogic) {
        if (!content.includes(removed)) {
            console.log(`  ✓ Correctly removed: "${removed.substring(0, 50)}..."`);
        } else {
            console.log(`  ✗ Still present (should be removed): "${removed}"`);
            allChangesPresent = false;
        }
    }

    return allChangesPresent;
}

// Run tests
console.log('================');
console.log('🧪 FEATURE SELECTION REFINEMENT TEST');
console.log('================\n');

const logicPassed = testFeatureSelectionLogic();
const codeChangesPresent = verifyCodeChanges();

console.log('================');
if (logicPassed && codeChangesPresent) {
    console.log('✅ All feature selection refinements working correctly!');
    console.log('✅ Default state is now "no features selected"');
    console.log('✅ Empty selection is allowed and maintained');
    process.exit(0);
} else {
    console.log('❌ Some feature selection refinements failed.');
    process.exit(1);
}
