# CLAUDE.md

This file provides guidance to coding agents when working with code in this repository.

## What this project is

A VS Code extension for Rust/Cargo development. It adds project configuration controls, a workspace target browser, cargo-make task management, and xtask alias support on top of rust-analyzer.

## Tooling

**Code navigation and editing**: Use the LSP tool for symbol lookup, go-to-definition, find-references, hover info, and call hierarchies. Do not use Serena MCP tools.

## Development commands

All project verification workflows go through the `xtask` binary via Cargo
aliases defined in `.cargo/config.toml`. Run the full verification suite only
when changes affect package or build setup, Rust code, or the development
environment (including CI, toolchain, and development tooling configuration).
Documentation-only changes, such as README, release-guide, or badge edits, do
not require these checks. When verification is required, use these commands:

| Command         | What it does                                      |
| --------------- | ------------------------------------------------- |
| `cargo compile` | Build WASM + bundle TypeScript (webpack)          |
| `cargo lint`    | `cargo clippy` + `cargo fmt --check` + ESLint     |
| `cargo xt-test` | Rust tests on native and `wasm32-unknown-unknown` |
| `cargo xt-pkg`  | Produce `cargo-tools.vsix`                        |

Run a single native Rust test: `cargo test <test_name>` inside the relevant package directory.

Run wasm tests only: `cargo test --target wasm32-unknown-unknown`

`wasm-bindgen-test-runner` is configured as the wasm32 test runner in `.cargo/config.toml`.

## Preparing a release

Follow `RELEASING.md` for the complete publishing process. When asked to
prepare version `<version>`:

1. Review all changes since the previous version tag. Add a dated
   `CHANGELOG.md` section for `<version>` and a comparison link from the
   previous tag.
2. Set `<version>` in `package.json`, both root-version entries in
   `package-lock.json`, `packages/cargo_tools/Cargo.toml`,
   `packages/cargo_tools_vscode/Cargo.toml`, and the matching first-party
   entries in `Cargo.lock`.
3. Update the version-specific npm and Git tag examples in `RELEASING.md`.
4. Run `npm ci`, followed by the full verification suite listed above.
5. Confirm the generated `cargo-tools.vsix` contains the expected publisher,
   extension name, version, and changelog. Also verify that repository-only
   files such as `AGENTS.md` are absent.
6. Review `git diff --check` and confirm that only release-preparation files
   changed.

Preparing a release does not authorize creating or pushing the version tag,
publishing the extension, or creating a GitHub release. Perform those actions
only when explicitly requested. Commit preparation changes only when requested,
using `chore: prepare for <version> release` with a concise commit body.

## Git commits

Always use [Conventional Commits](https://www.conventionalcommits.org/) for
commit messages, following the `<type>[optional scope]: <description>` format.
For non-trivial changes, include a concise commit body that motivates the
change and briefly describes how the resulting behavior was achieved.

## Architecture

This is a three-layer hybrid Rust/TypeScript project:

### Layer 1 — TypeScript (`vscode_extension/src/`)
A thin VS Code extension entry point. `extension.ts` activates, calls `run(workspaceFolder)` on the WASM module, and returns an `ExitToken`. `runtime.ts` exports VS Code API bindings (file I/O, task execution, logging, state persistence) that the Rust WASM code calls back into.

### Layer 2 — Rust/WASM (`packages/cargo_tools_vscode/`)
The core extension logic, compiled to `cdylib` targeting `wasm32-unknown-unknown`. Almost all code is guarded by `#[cfg(target_arch = "wasm32")]`. The entry point is `extension::run()` (exported via `wasm_bindgen`), which starts an `iced_viewless` async event loop with a top-level `Extension` state.

All components follow the iced/Elm pattern: `update(state, message) -> Task<Message>`. Components communicate via `Event` types returned alongside tasks (not directly via messages) to avoid coupling.

The `commands/` module is **not** gated on wasm32 — it exists on all targets so that `tests/command_consistency.rs` can compile natively and verify that every command ID in `package.json` has a corresponding implementation. Command IDs are defined in `commands/configuration.rs`, `commands/outline.rs`, `commands/cargo_make.rs`, `commands/pinned.rs`, `commands/xtask.rs`, and `commands/tasks.rs` (shared filter); `commands/cargo.rs` re-exports from the first two so the test can import from a single module.

`extension/vscode_task_utils.rs` contains the shared infrastructure for wiring up VS Code command handlers: `CommandBinding`/`OnFileChanged` type aliases, `send_file_changed`, `take_first`/`take_first_two` argument deserializers, and the generic `register_commands<Cmd, const N>` function. Each `extension/**/command.rs` only needs to define its `Command` enum and a `Command::all()` const array of `(&'static str, fn(Array) -> Option<Command>)` pairs, then call `register_commands(tx, Command::all())`.

### Layer 3 — Rust library (`packages/cargo_tools/`)
Platform-agnostic core logic; no VS Code or WASM dependencies. The WASM layer's `Extension` state owns two components backed by this library:

- **`Workspace`** (via `cargo::metadata`, `cargo::config`) — watches `Cargo.toml` manifests for changes, parses `cargo metadata` JSON into typed `Metadata`/`Package`/`Target` structs (also reads profile names), and delegates to `Configuration` (build profile, target, feature selection) and `Outline` (project tree view). `cargo::command` builds the typed `Command` enum into CLI arg lists.
- **`Tasks`** (via `cargo_make` and `xtask`) — parses `Makefile.toml` and delegates to `CargoMake` (task list/filtering) and `Pinned` (pinned task shortcuts); also reads `[alias]` entries from `.cargo/config.toml` via `xtask` and delegates to `Xtask` (alias list/filtering). This covers the [xtask pattern](https://github.com/matklad/cargo-xtask) where a workspace member is invoked as a build tool through a cargo alias. The `Tasks` top-level component also registers two shared VS Code commands (`commands/tasks.rs`) — "Filter by Name" and "Clear All Filters" — that apply to both the cargo-make and alias sub-panels simultaneously. `Pinned` receives both `MakefileTasks` and `XtaskAliases` from the top level so its "Add" picker can present both types in a single quick-pick.

`process` contains the `Process`/`CargoTaskContext` types used to pass executable processes across the WASM boundary.

### `iced_viewless` (`packages/iced_viewless/`)
A custom iced runtime that runs without any window or UI. Provides `application()` / `async_application()` builder APIs (identical to iced's but headless). Used by `cargo_tools_vscode` to drive the async event loop inside the VS Code extension host (which is already an async JavaScript environment).

## Key invariants

- **wasm32 boundary**: `cargo_tools_vscode` calls back into TypeScript only through `vs_code_api.rs` (which re-exports wasm-bindgen bindings generated from `vscode_extension/src/`). Never call VS Code APIs directly from `cargo_tools`.
- **Test portability**: Tests in `cargo_tools` use `#[wasm_bindgen_test(unsupported = test)]`, making them runnable on both native (`cargo test`) and wasm32 (`cargo test --target wasm32-unknown-unknown`).
- **Command consistency**: Adding a new VS Code command requires: (1) adding a string constant to the appropriate `commands/*.rs` file, (2) registering it in `package.json`, (3) adding an entry to `Command::all()` in the matching `extension/**/command.rs`, and (4) adding a variant to that file's `Command` enum and handling it in the component's `update()`.
