# Cargo Tools

![Cargo Tools](https://img.shields.io/badge/cargo--tools-VS%20Code%20Extension-blue)
![Build Status](https://github.com/NickelWenzel/cargo-tools/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-green)

This extension provides intuitive and advanced IDE-like features for Rust/Cargo development as a complement to the [rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Getting Started

1. Install [rustup](https://rustup.rs).
2. Install the [cargo-tools extension](https://github.com/NickelWenzel/cargo-tools).
3. **Open a Rust Project**: The extension automatically activates when it detects a `Cargo.toml` file.

## Features

### 🎯 Project configuration
- **Workspace Information**: Active project and Cargo.toml quick access
- **Build Configuration**: Current profile and default target selection
- **Quick Actions**: Build, Run, Test, and Bench commands with one click

### 📋 Project outline
- **Organized Target Structure**: Targets grouped by workspace member or type
- **Visual Indicators**: Icons and descriptions for different target types (bin, lib, example, test, bench)
- **Direct File Access**: Click any target to open its source file
- **Context Menus**: Right-click targets for build, run, test, debug actions
- **Advanced Filtering**: Filter by workspace member and target type with real-time preview
- **Customizable Grouping**: Toggle between workspace member and target type organization

### 🔧 Makefile support
- **cargo-make Integration**: Automatic detection and execution of `Makefile.toml` tasks
- **Task Categories**: Organized task structure with category-based grouping
- **Task Filtering**: Filter tasks by name or category for large makefiles
- **Pinned Tasks**: Pin frequently used tasks for quick access with keyboard shortcuts
- **One-Click Execution**: Run tasks directly from the tree view with integrated buttons

## Commands

### Quick Actions
- **Build** (`cargo-tools.projectStatus.build`) - `F7`: Build current target
- **Run** (`cargo-tools.projectStatus.run`) - `Ctrl+Shift+F5`: Run current target
- **Debug** (`cargo-tools.projectStatus.debug`) - `Shift+F5`: Debug current target
- **Test** (`cargo-tools.executeTestAction`): Run tests
- **Bench** (`cargo-tools.executeBenchAction`): Run benchmarks

### Target Management
- **Select Build Profile** (`cargo-tools.selectProfile`): Switch between dev/release
- **Select Build Target** (`cargo-tools.selectBuildTarget`): Choose default target
- **Select Run Target** (`cargo-tools.selectRunTarget`): Choose default run target
- **Select Benchmark Target** (`cargo-tools.selectBenchmarkTarget`): Choose default benchmark target
- **Select Platform Target** (`cargo-tools.selectPlatformTarget`): Choose target platform
- **Select Features** (`cargo-tools.selectFeatures`): Enable/disable cargo features

### Workspace Commands
- **Refresh** (`cargo-tools.refresh`): Reload workspace and targets
- **Clean** (`cargo-tools.clean`): Clean build artifacts
- **Build Documentation** (`cargo-tools.buildDocs`): Generate cargo docs

### Makefile Commands
- **Run Makefile Task** (`cargo-tools.makefile.runTask`): Execute selected makefile task
- **Filter Tasks** (`cargo-tools.makefile.setTaskFilter`): Filter makefile tasks by name
- **Filter Categories** (`cargo-tools.makefile.showCategoryFilter`): Filter by task category
- **Pin Task** (`cargo-tools.makefile.pinTask`): Pin task for quick access

### Pinned Task Shortcuts
- **Execute 1st - 5th Pinned Task** - `Ctrl+Alt+(1 - 5)`: Run 1st - 5th pinned makefile task

## Context Menus

Right-click items in different views for context-sensitive actions:

### Project Outline View
**Workspace Members/Packages:**
- **Select Package**: Set as active package for builds
- **Unselect Package**: Remove package selection
- **Build Package**: Build all targets in the package
- **Test Package**: Run all tests in the package
- **Clean Package**: Clean package build artifacts

**Workspace Root:**
- **Build Workspace**: Build entire workspace
- **Test Workspace**: Run all tests in workspace
- **Clean Workspace**: Clean all workspace build artifacts

**Targets (bin, lib, example, test, bench):**
- **Set as Build Target**: Use this target for build operations
- **Set as Run Target**: Use this target for run operations (executables only)
- **Set as Benchmark Target**: Use this target for benchmark operations
- **Build Target**: Build this specific target
- **Run Target**: Execute this target (executables only)
- **Debug Target**: Start debugging session for this target
- **Benchmark Target**: Run benchmarks for this target

### Makefile View
**Makefile Tasks:**
- **Run Task**: Execute the selected makefile task
- **Pin Task**: Add task to pinned tasks for quick access

### Pinned Makefile Tasks View
**Pinned Tasks:**
- **Execute Task**: Run the pinned task
- **Remove Task**: Remove from pinned tasks list

## Extension Settings

Configure the extension behavior by adding these settings to your VS Code `settings.json`:

### Basic Configuration
```json
{
  "cargoTools.defaultProfile": "dev",           // Default build profile: "dev" or "release"
  "cargoTools.cargoCommand": "cargo",           // Cargo command or wrapper to use
  "cargoTools.cargoPath": "cargo"               // Path to cargo executable
}
```

### Feature Configuration
```json
{
  "cargoTools.features": ["feature1", "feature2"],  // Default features to enable
  "cargoTools.noDefaultFeatures": false             // Disable default features
}
```

### Build and Run Arguments
```json
{
  "cargoTools.buildArgs": ["--verbose"],               // Additional cargo build arguments
  "cargoTools.run.extraArgs": ["--", "arg1", "arg2"], // Extra arguments for run/debug
  "cargoTools.test.extraArgs": ["--verbose"]          // Extra arguments for test/bench
}
```

### Environment Variables
```json
{
  "cargoTools.extraEnv": {                          // Environment variables for all commands
    "RUST_LOG": "debug",
    "CUSTOM_VAR": "value"
  },
  "cargoTools.run.extraEnv": {                      // Additional env vars for run/debug
    "RUN_MODE": "debug"
  },
  "cargoTools.test.extraEnv": {                     // Additional env vars for test/bench
    "TEST_MODE": "verbose"
  }
}
```

### Command Overrides
```json
{
  "cargoTools.runCommandOverride": "cargo watch -x run",  // Override for run command
  "cargoTools.testCommandOverride": "cargo nextest run"   // Override for test command
}
```

### rust-analyzer Integration
```json
{
  "cargoTools.useRustAnalyzerEnvAndArgs": false,    // Use rust-analyzer settings
  "cargoTools.updateRustAnalyzerTarget": false      // Auto-update rust-analyzer target
}
```

### Legacy Settings (Deprecated)
```json
{
  "cargoTools.runArgs": [],        // Use cargoTools.run.extraArgs instead
  "cargoTools.testArgs": [],       // Use cargoTools.test.extraArgs instead  
  "cargoTools.environment": {}     // Use cargoTools.extraEnv instead
}
```

## Requirements

- **Visual Studio Code** 1.102.0 or higher
- **Rust toolchain** with Cargo installed
- **cargo** command available in PATH (or configure `cargoTools.cargoPath`)

## Known Issues

- Large workspaces may take a moment to parse on first load
- Debug support requires [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
- Icon and color choice may be improved

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## Inspiration

This extension draws inspiration from [vscode-cmake-tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cmake-tools).

## License

MIT License - see LICENSE file for details.
