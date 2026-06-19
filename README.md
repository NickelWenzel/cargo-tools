# Cargo Tools

![Build Status](https://github.com/NickelWenzel/cargo-tools/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-green)

Cargo Tools is a Visual Studio Code extension that provides IDE-like features for Rust/Cargo development. It complements [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) by adding project configuration controls, a workspace target browser, cargo-make task management, and cargo alias shortcuts.

## Features

### Project Configuration

![Project configuration](./vscode_extension/media/cargo_tools_configuration.gif)

* Select defaults for target platform, cargo pprofile, package
* Select default build run, benchmarks targets and features for packages
* Trigger cargo through user interface or key bindings
* Build documention
* Install additional targt platforms, clean buid artifacts

### Project Outline

![Project outline](./vscode_extension/media/cargo_tools_project_outline.gif)

* Overview over workspace
* Show available targets for workspace members
* Build, run, debug, test and clean workspace, packages and targets
* Filter by package name and target type
* Group by package or target type

### [xtask](https://github.com/matklad/cargo-xtask)/alias and cargo-make Integration

![cargo make](./vscode_extension/media/cargo_tools_makefile_support.gif)

* Overview over available cargo aliases and cargo make tasks
* Run tasks with or without additional arguments
* Filter by name or task category
* Pin default tasks and trigge them through key bindings

## Usage

### Requirements

- Visual Studio Code 1.102.0 or higher
- Rust toolchain with Cargo installed via [rustup](https://rustup.rs)

### Installation

Launch VS Code Quick Open (Ctrl+P), paste the following command, and press enter.
```bash
ext install NickelWenzel.cargo-tools
```

### Resources

* [Docs](./vscode_extension/docs/README.md) for comprehensive documentation.
* [Github](https://github.com/NickelWenzel/cargo-tools) for source code, issues, and contributing.

## Development

### Prerequisites
* Rust stable
* Node.js 20
* wasm-pack 0.14
* wasm-bindgen-cli 0.2.106

Development workflows through `xtask`s defined in `.cargo/config.toml`.

| Command         | Description                                           |
| --------------- | ----------------------------------------------------- |
| `cargo compile` | Build WASM + bundle TypeScript                        |
| `cargo lint`    | `cargo clippy` + `cargo fmt --check` + ESLint         |
| `cargo xt-test` | Run Rust tests on native and `wasm32-unknown-unknown` |
| `cargo xt-pkg`  | Produce `cargo-tools.vsix`                            |

## Contributing

Contributions welcome via PRs or issues on [Github](https://github.com/NickelWenzel/cargo-tools).

## Inspiration

This extension draws inspiration from [vscode-cmake-tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cmake-tools).

## License

MIT
