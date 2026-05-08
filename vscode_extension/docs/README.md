# Cargo Tools Documentation

- [Commands Reference](commands.md) — Complete list of all commands
- [Settings Reference](settings.md) — All available settings with types and defaults

# Getting Started

This page walks you through installing Cargo Tools and opening your first Rust project.

## Prerequisites

- **Visual Studio Code** 1.102.0 or higher
- **Rust toolchain** with Cargo installed via [rustup](https://rustup.rs)
- `cargo` available in `PATH` (or configured via [`cargoTools.cargoCommand`](settings.md#cargo-invocation))

## Installation

Install Cargo Tools from the Visual Studio Code Marketplace by searching for *Cargo Tools* in the Extensions view (`Ctrl+Shift+X`), or download a `.vsix` release and install it with **Extensions: Install from VSIX...**.

## Activation

The extension activates automatically when VS Code detects a `Cargo.toml` file in the open workspace. No manual activation is required.

Once active, the following views appear in the **Cargo Tools** Activity Bar panel:

| View                      | Description                                                                                |
| ------------------------- | ------------------------------------------------------------------------------------------ |
| **Configuration**         | Active profile, package, and target selections with quick-action buttons                   |
| **Project Outline**       | Hierarchical tree of workspace members, packages, and targets                              |
| **Makefile**              | Tasks from `Makefile.toml` — visible only when a `Makefile.toml` is present                |
| **Pinned Makefile Tasks** | Pinned tasks for keyboard-shortcut access — visible only when a `Makefile.toml` is present |

A condensed **Cargo Tools** panel also appears in the Explorer sidebar.

## First Steps

1. Open a Rust project folder containing a `Cargo.toml` file.
2. Click the **Cargo Tools** icon in the Activity Bar.
3. In the **Configuration** view, select a build profile (default: `dev`) and a run target.
4. Press `F7` to build, `Ctrl+Shift+F5` to run, or `Shift+F5` to debug.

