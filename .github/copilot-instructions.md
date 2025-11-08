# Copilot Coding Agent Onboarding Instructions for cargo-tools

## Repository Overview

- **Purpose:** This repository is a Visual Studio Code extension for Rust development, providing advanced workspace, build, run, test, and benchmark management for Cargo-based Rust projects. It is similar in spirit to the cmake-tools extension but targets Rust and Cargo workflows.
- **Type:** VS Code extension (TypeScript + Rust/WASM)
- **Languages:** Rust (core logic), TypeScript (VS Code integration), JavaScript (tests), JSON (configuration)
- **Size:** Medium (Rust core in `packages/`, TypeScript integration in `src/`, test suite, sample Rust project)
- **Target Runtime:** Node.js (VS Code extension host) with WebAssembly modules

## Build, Test, and Validation Instructions

### Environment Setup
- **Node.js:** Use Node.js v18+ (recommended)
- **Rust:** Required for WASM module compilation (stable toolchain)
- **cargo-make:** Required for build automation (`cargo install cargo-make`)
- **wasm-pack:** Required for WASM bindings generation (`cargo install wasm-pack`)
- **VS Code:** Latest stable version recommended
- **Always run `npm install` before building or testing.**

### Build Steps
- **Compile the extension (Rust + TypeScript):**
  ```sh
  cargo make compile
  ```
  - Builds Rust WASM modules using wasm-pack from `packages/cargo_tools_vscode/`
  - Generates TypeScript bindings in `src/wasm/`
  - Uses Webpack to bundle TypeScript sources and WASM modules into `extension.js`
  - Output is placed in the default VS Code extension output location
  - If you see errors, ensure Rust toolchain, wasm-pack, and all dependencies are installed

### Test Steps
- **Run the full test suite:**
  ```sh
  cargo make test
  ```
  - Compiles Rust WASM modules
  - Runs TypeScript compilation, lint, and all integration/unit tests using `vscode-test`
  - Tests are located in `src/test/` and cover command registration, argument generation, WASM integration, and extension logic
  - If tests fail, check for missing dependencies, Rust compilation errors, or TypeScript errors

### Linting
- **Lint the codebase:**
  ```sh
  cargo make lint
  ```
  - Runs ESLint on TypeScript code with config in `eslint.config.mjs`
  - Runs clippy on Rust code for additional static analysis
  - Run after making changes to ensure code style and correctness

### Package Extension
- **Create extension package:**
  ```sh
  cargo make package
  ```
  - Builds optimized WASM modules and TypeScript bundle
  - Creates `.vsix` extension package for distribution

### Clean Build
- **To ensure a clean build:**
  ```sh
  rm -rf out/ dist/ src/wasm/ target/ && cargo make compile
  ```
  - Removes previous build artifacts including WASM outputs and recompiles

### Common Issues & Workarounds
- If you see Rust compilation errors, ensure the stable toolchain is installed and up to date
- If wasm-pack fails, ensure it's installed and PATH is configured correctly
- If WASM binding generation fails, check the Rust lib.rs exports and wasm_bindgen configuration
- If TypeScript or Webpack errors occur, always run `npm install` first
- If VS Code does not recognize the extension, reload the window or restart VS Code
- If tests hang or fail due to environment, ensure you are not running in a restricted container and that you have write access to the workspace
- If you see errors about missing VS Code APIs, update VS Code and dependencies

## Project Layout & Architecture

- **Root Files:**
  - `Cargo.toml`: Rust workspace configuration defining WASM packages
  - `Makefile.toml`: cargo-make task definitions for build automation
  - `package.json`: Extension manifest, scripts, command registration
  - `tsconfig.json`: TypeScript configuration
  - `webpack.config.js`: Webpack bundling config with WASM support
  - `eslint.config.mjs`: ESLint config
  - `README.md`: Project overview and usage
  - `.github/copilot-instructions.md`: (this file)
- **Rust Source Directory (`packages/cargo_tools_vscode/`):**
  - Contains core business logic implemented in Rust with specific architectural patterns:
    - **`vs_code_api` module:** All methods and types imported from TypeScript must reside in this module
    - **`cargo_tools_ext` module:** Contains the `CargoToolsExt` struct, which is a thin wrapper around the core `CargoTools` type
    - **Trait-based abstraction:** All functions or types requiring VS Code API access must be abstracted by traits
    - **Concrete implementations:** Only concrete trait implementations should use the `vs_code_api` module
    - **WASM bindings:** TypeScript bindings are generated via wasm_bindgen for the `CargoToolsExt` interface
  - Files will evolve during implementation - check current directory structure
  - Focus on lib.rs for main exports and module organization
- **TypeScript Source Directory (`src/`):**
  - **Migration Target:** All TypeScript logic should ideally be moved to Rust/WASM for performance and maintainability
  - Contains VS Code extension integration code and WASM bindings
  - File structure will change as code migrates to Rust - avoid assumptions about specific files
  - `wasm/`: Generated TypeScript bindings and WASM modules (auto-generated, do not edit)
  - `test/`: Test suites covering extension functionality and WASM integration
  - Main entry point and VS Code API integration remain in TypeScript temporarily
- **Sample Rust Project:**
  - `test-rust-project/`: Used for integration testing

## Validation & CI
- **Tests:** Always run `cargo make test` before committing changes. All tests must pass.
- **Lint:** Run `cargo make lint` before check-in.
- **Build:** Run `cargo make compile` to ensure both Rust WASM modules and TypeScript build.
- **No explicit GitHub Actions or CI pipeline is present, but local validation is required.**

## Key Facts for Efficient Agent Work
- **Hybrid Architecture:** Core logic is implemented in Rust (performance-critical operations) with TypeScript handling VS Code integration
- **WASM Integration:** Rust code is compiled to WebAssembly and imported into TypeScript via generated bindings
- **Migration Strategy:** Business logic should be moved from TypeScript to Rust incrementally
- **Architectural Constraints:**
  - All TypeScript imports in Rust code must be defined in the `vs_code_api` module
  - The `CargoToolsExt` struct serves as the primary WASM interface, wrapping the core `CargoTools` functionality
  - wasm_bindgen generates TypeScript bindings specifically for the `CargoToolsExt` interface
  - **Trait Abstraction:** All functions or types requiring VS Code API access must be abstracted by traits
  - **Implementation Isolation:** Only concrete trait implementations should depend on the `vs_code_api` module
  - **Test Independence:** Tests use mock implementations of traits that do not depend on `vs_code_api`
- **Command Registration:** Extension commands are registered via VS Code API in TypeScript
- **Task Execution:** Cargo command execution should be handled in Rust when possible
- **State Management:** Core state logic should be implemented in Rust with TypeScript wrappers
- **Testing Strategy:** Logic tests use mock trait implementations to avoid VS Code API dependencies
- **Configuration:** All extension settings are under the `cargoTools` namespace in `package.json`
- **Build System:** Uses cargo-make for orchestrating Rust compilation, WASM generation, and TypeScript bundling

## Agent Guidance
- **Trust these instructions for build, test, and validation.** Only search the codebase if information here is incomplete or found to be in error.
- **Always run `npm install` and ensure Rust toolchain is installed before any build or test.**
- **Use cargo-make commands:** `cargo make compile`, `cargo make lint`, and `cargo make test` for all validation.
- **For Rust changes:**
  - Modify files in `packages/cargo_tools_vscode/src/` and ensure proper wasm_bindgen exports
  - Place all TypeScript imports in the `vs_code_api` module
  - Use `CargoToolsExt` as the primary interface for WASM bindings
  - Ensure `CargoToolsExt` properly wraps `CargoTools` functionality
  - **Abstract VS Code API dependencies behind traits**
  - **Keep trait implementations separate from core logic**
  - **Use mock implementations for testing that don't depend on VS Code API**
- **For TypeScript changes:** Consider migrating logic to Rust instead when possible. For VS Code integration code, modify files in `src/` and ensure WASM integration points are maintained.
- **For UI or command changes:** Update both `package.json` and the relevant implementation files.
- **For new features:** 
  - Implement core logic in Rust using the `CargoTools` type with trait abstractions
  - Create traits for any VS Code API dependencies
  - Implement concrete trait implementations in separate modules that use `vs_code_api`
  - Expose functionality through the `CargoToolsExt` wrapper with wasm_bindgen
  - Add TypeScript wrappers if needed for VS Code integration
  - **Add tests using mock trait implementations that don't depend on VS Code API**
  - Add tests covering both Rust and TypeScript layers
- **For testing:**
  - Create mock implementations of traits for unit testing
  - Ensure tests can run without VS Code API dependencies
  - Test core logic independently from VS Code integration
- **WASM Workflow:** Rust changes require rebuilding WASM modules, which regenerates TypeScript bindings in `src/wasm/`.
- **If you encounter errors:** Check for missing Rust toolchain, wasm-pack installation, Node.js dependencies, outdated VS Code, or TypeScript issues.
- **Documentation Updates:** If changes make this copilot-instructions.md file outdated or inaccurate, propose updates to keep it current and useful.

---

This onboarding file is intended to minimize exploration and maximize the efficiency and reliability of Copilot coding agent work. If you find any information here to be incomplete or incorrect, perform a targeted search and update this file as needed.
