# Cargo Tools

![Build Status](https://github.com/NickelWenzel/cargo-tools/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-green)

Cargo Tools is a Visual Studio Code extension that provides IDE-like features for Rust/Cargo development. It complements [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) by adding project configuration controls, a workspace target browser, and cargo-make task management.

## Features

- **[Project Configuration](docs/project-configuration.md)** — Select build profiles, packages, platform targets, and features; run build, run, debug, test, and bench actions from the sidebar
- **[Project Outline](docs/project-outline.md)** — Browse workspace members and targets in a filterable, groupable tree; run direct cargo actions on any target without changing your active selection
- **[cargo-make Integration](docs/cargo-make.md)** — Discover, filter, and run `Makefile.toml` tasks from a dedicated view; pin frequently used tasks for one-keystroke access

## Documentation

- [Online documentation](https://github.com/NickelWenzel/cargo-tools/packages/cargo_tools_vscode/extension/README.md)

## Requirements

- Visual Studio Code 1.102.0 or higher
- Rust toolchain with Cargo installed via [rustup](https://rustup.rs)

## Inspiration

This extension draws inspiration from [vscode-cmake-tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cmake-tools).

## License

MIT — see [LICENSE](LICENSE) for details.
