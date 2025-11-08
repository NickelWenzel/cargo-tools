#!/bin/bash
# Verification script for StateValue derive macro implementation

set -e

echo "=== StateValue Derive Macro Verification ==="
echo ""

echo "Step 1: Testing macro crate..."
cargo test -p cargo_tools_macros
echo "✓ Macro tests passed"
echo ""

echo "Step 2: Checking workspace builds..."
cargo check --workspace
echo "✓ Workspace builds successfully"
echo ""

echo "Step 3: Running clippy on macro crate..."
cargo clippy -p cargo_tools_macros -- -D warnings
echo "✓ Macro passes clippy"
echo ""

echo "Step 4: Running clippy on main crate..."
cargo clippy -p cargo_tools_vscode -- -D warnings
echo "✓ Main crate passes clippy"
echo ""

echo "Step 5: Building extension..."
cargo make compile
echo "✓ Extension compiled successfully"
echo ""

echo "Step 6: Running full test suite..."
cargo make test
echo "✓ All tests passed"
echo ""

echo "Step 7: Running lint..."
cargo make lint
echo "✓ Lint passed"
echo ""

echo "=== All verification steps completed successfully! ==="
