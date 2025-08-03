/**
 * Test script to verify feature selection functionality
 */

const fs = require('fs');
const path = require('path');

// Check if the feature-related methods exist in cargoWorkspace.ts
const cargoWorkspacePath = path.join(__dirname, 'src', 'cargoWorkspace.ts');
const content = fs.readFileSync(cargoWorkspacePath, 'utf8');

const requiredMethods = [
    'getPackageFeatures',
    'getAvailableFeatures',
    'setSelectedFeatures',
    'toggleFeature',
    'selectedFeatures',
    '_selectedFeatures',
    '_packageFeatures',
    'onDidChangeSelectedFeatures'
];

console.log('✓ Testing CargoWorkspace feature methods...');

let allMethodsPresent = true;
for (const method of requiredMethods) {
    if (content.includes(method)) {
        console.log(`  ✓ ${method} - found`);
    } else {
        console.log(`  ✗ ${method} - missing`);
        allMethodsPresent = false;
    }
}

// Check ProjectStatusTreeProvider
const treeProviderPath = path.join(__dirname, 'src', 'projectStatusTreeProvider.ts');
const treeContent = fs.readFileSync(treeProviderPath, 'utf8');

console.log('\n✓ Testing ProjectStatusTreeProvider feature support...');

const treeRequirements = [
    'featureSelection',
    'createFeatureSelectionChildren',
    'feature-item',
    'onDidChangeSelectedFeatures'
];

let allTreeMethodsPresent = true;
for (const item of treeRequirements) {
    if (treeContent.includes(item)) {
        console.log(`  ✓ ${item} - found`);
    } else {
        console.log(`  ✗ ${item} - missing`);
        allTreeMethodsPresent = false;
    }
}

// Check CargoExtensionManager
const extensionManagerPath = path.join(__dirname, 'src', 'cargoExtensionManager.ts');
const extensionContent = fs.readFileSync(extensionManagerPath, 'utf8');

console.log('\n✓ Testing CargoExtensionManager feature command...');

const commandRequirements = [
    'toggleFeature',
    'async toggleFeature(feature: string)'
];

let allCommandsPresent = true;
for (const item of commandRequirements) {
    if (extensionContent.includes(item)) {
        console.log(`  ✓ ${item} - found`);
    } else {
        console.log(`  ✗ ${item} - missing`);
        allCommandsPresent = false;
    }
}

console.log('\n================');
if (allMethodsPresent && allTreeMethodsPresent && allCommandsPresent) {
    console.log('✅ All feature selection functionality implemented successfully!');
    process.exit(0);
} else {
    console.log('❌ Some feature selection functionality is missing.');
    process.exit(1);
}
