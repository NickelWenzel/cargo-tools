# Development Commands

## Build Commands
- `cargo make compile` - Compile extension (Rust WASM + TypeScript)
- `cargo build -p cargo_tools` - Build cargo_tools package (native)
- `cargo build -p cargo_tools --target wasm32-unknown-unknown` - Build for WASM

## Test Commands
- `cargo make test` - Run full test suite (compiles WASM + runs tests)
- `cargo test -p cargo_tools` - Run tests for cargo_tools package
- `cargo test -p cargo_tools_macros` - Run tests for macros package

## Linting & Formatting
- `cargo make lint` - Run ESLint + clippy
- `cargo clippy` - Run clippy on Rust code
- `cargo fmt` - Format Rust code

## Package & Clean
- `cargo make package` - Create .vsix extension package
- `rm -rf out/ dist/ src/wasm/ target/ && cargo make compile` - Clean build

## Important Notes
- Always run `npm install` before building/testing
- cargo_tools_vscode is WASM-only (wasm32-unknown-unknown target)
- Other crates must compile for both WASM and native targets
- Tests run on native target only
