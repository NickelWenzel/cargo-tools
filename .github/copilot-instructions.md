
# Copilot Coding Agent Onboarding Instructions for cargo-tools

## Repository Overview

- **Purpose:** This repository is a Visual Studio Code extension for Rust development, providing advanced workspace, build, run, test, and benchmark management for Cargo-based Rust projects. It is similar in spirit to the cmake-tools extension but targets Rust and Cargo workflows.
- **Type:** VS Code extension (TypeScript)
- **Languages:** TypeScript (main), JavaScript (tests), JSON (configuration)
- **Size:** Medium (core logic in `src/`, test suite, sample Rust project)
- **Target Runtime:** Node.js (VS Code extension host)

## Build, Test, and Validation Instructions

### Environment Setup
- **Node.js:** Use Node.js v18+ (recommended)
- **VS Code:** Latest stable version recommended
- **Rust:** Not required for extension build/test, but useful for validating Rust project integration
- **Always run `npm install` before building or testing.**

### Build Steps
- **Compile the extension:**
  ```sh
  npm run compile
  ```
  - Uses Webpack to bundle TypeScript sources from `src/` into `extension.js`.
  - Output is placed in the default VS Code extension output location.
  - If you see errors, ensure all dependencies are installed and TypeScript is available.

### Test Steps
- **Run the full test suite:**
  ```sh
  npm test
  ```
  - Runs compile, lint, and all integration/unit tests using `vscode-test`.
  - Tests are located in `src/test/` and cover command registration, argument generation, and integration logic.
  - If tests fail, check for missing dependencies or TypeScript errors.

### Linting
- **Lint the codebase:**
  ```sh
  npm run lint
  ```
  - Uses ESLint with config in `eslint.config.mjs`.
  - Run after making changes to ensure code style and correctness.

### Clean Build
- **To ensure a clean build:**
  ```sh
  rm -rf out/ dist/ && npm run compile
  ```
  - Removes previous build artifacts and recompiles.

### Common Issues & Workarounds
- If you see TypeScript or Webpack errors, always run `npm install` first.
- If VS Code does not recognize the extension, reload the window or restart VS Code.
- If tests hang or fail due to environment, ensure you are not running in a restricted container and that you have write access to the workspace.
- If you see errors about missing VS Code APIs, update VS Code and dependencies.

## Project Layout & Architecture

- **Root Files:**
  - `package.json`: Extension manifest, scripts, command registration
  - `tsconfig.json`: TypeScript configuration
  - `webpack.config.js`: Webpack bundling config
  - `eslint.config.mjs`: ESLint config
  - `README.md`: Project overview and usage
  - `.github/copilot-instructions.md`: (this file)
- **Source Directory (`src/`):**
  - `extension.ts`: Main extension entry point
  - `cargoExtensionManager.ts`: Core singleton managing workspace, commands, and UI
  - `cargoWorkspace.ts`: Workspace and target discovery logic
  - `cargoTaskProvider.ts`: VS Code TaskProvider for cargo commands
  - `cargoProfile.ts`, `cargoTarget.ts`: Types and logic for profiles/targets
  - `projectOutlineTreeProvider.ts`, `statusBarProvider.ts`, `profilesTreeProvider.ts`, `targetsTreeProvider.ts`, `workspaceTreeProvider.ts`: UI components for tree/status bar views
  - `test/extension.test.ts`, `test/cargoExtensionManager.test.ts`: Test suites
- **Sample Rust Project:**
  - `test-rust-project/`: Used for integration testing

## Validation & CI
- **Tests:** Always run `npm test` before committing changes. All 51+ tests must pass.
- **Lint:** Run `npm run lint` before check-in.
- **Build:** Run `npm run compile` to ensure the extension builds.
- **No explicit GitHub Actions or CI pipeline is present, but local validation is required.**

## Key Facts for Efficient Agent Work
- **Command Registration:** All extension commands are registered in `cargoExtensionManager.ts` and listed in `package.json`.
- **UI Actions:** Tree view and status bar actions are managed in `src/projectOutlineTreeProvider.ts` and `src/statusBarProvider.ts`.
- **Task Execution:** All cargo command execution is routed through `cargoTaskProvider.ts`.
- **Tests:** All validation logic is in `src/test/` and covers command registration, argument generation, and integration.
- **Configuration:** All extension settings are under the `cargoTools` namespace in `package.json`.
- **Features:** Inline and context menu action buttons for build/run/test/bench are available in the project outline pane and status bar.

## Agent Guidance
- **Trust these instructions for build, test, and validation.** Only search the codebase if information here is incomplete or found to be in error.
- **Always run `npm install` before any build or test.**
- **Validate changes by running `npm run compile`, `npm run lint`, and `npm test` in that order.**
- **For UI or command changes, update both `package.json` and the relevant TypeScript files in `src/`.**
- **For new features, add tests in `src/test/` and validate with `npm test`.**
- **If you encounter errors, check for missing dependencies, outdated VS Code, or TypeScript issues.**

---

This onboarding file is intended to minimize exploration and maximize the efficiency and reliability of Copilot coding agent work. If you find any information here to be incomplete or incorrect, perform a targeted search and update this file as needed.
