# Cargo Tools

![Build Status](https://github.com/NickelWenzel/cargo-tools/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-green)

Cargo Tools is a Visual Studio Code extension that provides IDE-like features for Rust/Cargo development. It complements [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) by adding project configuration controls, a workspace target browser, and cargo-make task management.

## Usage

See the [docs](./packages/cargo_tools_vscode/extension/docs/README.md) for the full reference.

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


## Inspiration

This extension draws inspiration from [vscode-cmake-tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cmake-tools).

## License

MIT
