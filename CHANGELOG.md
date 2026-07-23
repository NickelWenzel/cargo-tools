# Changelog

All notable changes to Cargo Tools are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - Unreleased

### Added

- Task and xtask alias pickers now remember recently executed entries and place them at the top for quicker access.
- Pinned task executions also update the recent-task order.

### Improved

- Task name filters now update as a preview and are only persisted after selection, reducing unnecessary configuration writes.
- Workspace file changes are debounced to avoid repeated refreshes when several filesystem events arrive together.
- Cargo metadata handling, task filtering, sorting, and quick-pick logging have been optimized for a more responsive extension.
- Obsolete entries are automatically removed from the recent-task history when tasks or aliases no longer exist.
- Extension packaging no longer scans development dependencies that are not included in the bundled extension.

## [0.4.3] - 2026-07-04

### Fixed

- Profile changes now update the workspace UI and dependent selections reliably.
- Cargo Make tasks now refresh correctly when their Makefile changes.
- Missing workspace files are distinguished from command execution failures, resulting in clearer and more accurate error handling.
- Disabled an incompatible Tokio context configuration in WebAssembly builds.

### Improved

- Reduced noisy diagnostic logging during normal extension use.
- Cleaned up workspace dependencies and runtime integration.

## [0.4.2] - 2026-06-19

### Fixed

- Workspace outline filtering is now applied immediately after selecting a new filter.

### Improved

- Added an overview of the extension's available features to the README.
- Removed the obsolete repository-level Cargo Make configuration.
- Improved release and CI task configuration.

[0.5.0]: https://github.com/NickelWenzel/cargo-tools/compare/v0.4.3...HEAD
[0.4.3]: https://github.com/NickelWenzel/cargo-tools/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/NickelWenzel/cargo-tools/compare/v0.4.1...v0.4.2
