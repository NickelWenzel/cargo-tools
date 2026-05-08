# Cargo Tools

![Build Status](https://github.com/NickelWenzel/cargo-tools/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-green)

Cargo Tools is a Visual Studio Code extension that provides IDE-like features for Rust/Cargo development. It complements [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) by adding project configuration controls, a workspace target browser, and cargo-make task management.

## Features

### Project Configuration

![Project configuration](./vscode_extension/media/cargo_tools_configuration.gif)

### Project Outline

![Project outline](./vscode_extension/media/cargo_tools_project_outline.gif)

### cargo-make Integration

![cargo make](./vscode_extension/media/cargo_tools_makefile_support.gif)

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
* cargo-make
* wasm-pack 0.14
* wasm-bindgen-cli 0.2.106.

Development workflows are driven by `cargo make`.

| Command              | Description                                           |
| -------------------- | ----------------------------------------------------- |
| `cargo make compile` | Build WASM + bundle TypeScript                        |
| `cargo make lint`    | Run `cargo clippy` and ESLint                         |
| `cargo make test`    | Run Rust tests on native and `wasm32-unknown-unknown` |
| `cargo make package` | Produce `cargo-tools.vsix`                            |

## Contributing

Contributions welcome via PRs or issues on [Github](https://github.com/NickelWenzel/cargo-tools).

## Inspiration

This extension draws inspiration from [vscode-cmake-tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cmake-tools).

## License

MIT
