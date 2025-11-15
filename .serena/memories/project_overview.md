# Project Overview

## Purpose
Visual Studio Code extension for Rust development providing advanced workspace, build, run, test, and benchmark management for Cargo-based Rust projects. Similar to cmake-tools extension but for Rust/Cargo workflows.

## Tech Stack
- **Languages**: Rust (core logic), TypeScript (VS Code integration), JavaScript (tests)
- **Architecture**: Hybrid - Rust core compiled to WASM, TypeScript handles VS Code API integration
- **Runtime**: Node.js (VS Code extension host) with WebAssembly modules
- **Build Tools**: cargo-make, wasm-pack, webpack
- **Testing**: tokio::test, tracing-test, vscode-test

## Project Structure
- `packages/cargo_tools/`: Core Rust library (platform-independent)
- `packages/cargo_tools_macros/`: Procedural macros
- `packages/cargo_tools_vscode/`: WASM-specific crate for VS Code integration
- `src/`: TypeScript VS Code integration (migrating to Rust)
- `test-rust-project/`: Sample project for integration tests
