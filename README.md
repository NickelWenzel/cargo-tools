# Cargo Tools for Visual- **Easy Management**: Add and remove commands with simple UI

## Getting Started

1. **Install the Extension**: Search for "Cargo Tools" in VS Code Extensions
2. **Open a Rust Project**: The extension automatically activates when it detects a `Cargo.toml` filee

![Cargo Tools](https://img.shields.io/badge/cargo--tools-VS%20Code%20Extension-blue)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-green)

A comprehensive Visual Studio Code extension that provides advanced Rust/Cargo development tools with modern tree-based UI, build profiles, target management, and workspace integration.

## Features

### 🎯 Project Status View
- **Workspace Information**: Active project and Cargo.toml quick access
- **Build Configuration**: Current profile and default target selection
- **Quick Actions**: Build, Run, Test, and Bench commands with one click

### 📋 Project Outline View  
- **Organized Target Structure**: Targets grouped by workspace member or type
- **Visual Indicators**: Icons and descriptions for different target types (bin, lib, example, test, bench)
- **Direct File Access**: Click any target to open its source file
- **Context Menus**: Right-click targets for build, run, test, debug actions

## Getting Started

1. **Install the Extension**: Search for "Cargo Tools" in VS Code Extensions
2. **Open a Rust Project**: The extension automatically activates when it detects a `Cargo.toml` file
3. **Explore the Views**: Access the Cargo Tools views from the Activity Bar (package icon)

## UI Overview

The extension provides multiple tree views organized in the Cargo Tools sidebar:

### Project Status
Shows current workspace state and provides quick access to common actions:
- **Workspace** → Active project information
- **Build Configuration** → Profile and target selection  
- **Actions** → Build, Run, Test, Bench shortcuts

### Project Outline
Displays your project structure with two organization modes:
- **By Workspace Member** (default): Groups targets under each workspace package
- **By Target Type**: Groups all binaries, libraries, examples, etc. together

Configure the grouping with: `cargoTools.groupTargetsByWorkspaceMember`

## Configuration

### Settings

```json
{
  "cargoTools.defaultProfile": "dev",
  "cargoTools.cargoPath": "cargo",
  "cargoTools.buildArgs": [],
  "cargoTools.runArgs": [],
  "cargoTools.testArgs": [],
  "cargoTools.environment": {},
  "cargoTools.features": [],
  "cargoTools.allFeatures": false,
  "cargoTools.noDefaultFeatures": false,
  "cargoTools.groupTargetsByWorkspaceMember": true,
}
```

### Key Settings

- **`defaultProfile`**: Choose between "dev" and "release" build profiles
- **`features`**: Default features to enable for builds

## Commands

### Quick Actions
- **Build** (`cargo-tools.executeBuildAction`): Build current target
- **Run** (`cargo-tools.executeRunAction`): Run current target
- **Test** (`cargo-tools.executeTestAction`): Run tests
- **Bench** (`cargo-tools.executeBenchAction`): Run benchmarks

### Target Management
- **Select Build Profile** (`cargo-tools.selectProfile`): Switch between dev/release
- **Select Build Target** (`cargo-tools.selectTarget`): Choose default target
- **Set as Default Target** (`cargo-tools.setAsDefaultTarget`): Set target as default

### Workspace Commands
- **Refresh** (`cargo-tools.refresh`): Reload workspace and targets
- **Run Example** (`cargo-tools.runExample`): Quick example execution
- **Run Test** (`cargo-tools.runTest`): Specific test execution

## Context Menus

Right-click targets in any view for context-sensitive actions:

- **Build Target**: Compile the selected target
- **Run Target**: Execute binary/example targets
- **Test Target**: Run test targets  
- **Debug Target**: Start debugging session
- **Set as Default**: Make target the default for quick actions

## Task Integration

The extension automatically provides Cargo tasks for VS Code's task system. Configure tasks in `tasks.json`:

```json
{
  "type": "cargo",
  "command": "build",
  "profile": "release",
  "target": "my-binary",
  "features": ["feature1", "feature2"]
}
```

## Workspace Support

### Single Package Projects
- Targets organized by type (binaries, libraries, examples, tests, benchmarks)
- Simple project structure and navigation

### Multi-Package Workspaces  
- Targets grouped by workspace member (configurable)
- Package-aware command execution
- Automatic package context in cargo commands

## Keyboard Shortcuts

No default shortcuts are provided, but you can bind any command:

1. Open Command Palette (`Ctrl+Shift+P`)
2. Type "Preferences: Open Keyboard Shortcuts"
3. Search for "cargo-tools" commands
4. Assign your preferred shortcuts

## Requirements

- **Visual Studio Code** 1.102.0 or higher
- **Rust toolchain** with Cargo installed
- **cargo** command available in PATH (or configure `cargoTools.cargoPath`)

## Known Issues

- Large workspaces may take a moment to parse on first load
- Debug support requires appropriate debug extensions (e.g., CodeLLDB)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## Development

To set up the development environment:

```bash
# Clone the repository
git clone https://github.com/your-username/cargo-tools.git
cd cargo-tools

# Install dependencies
npm install

# Open in VS Code
code .

# Press F5 to run the extension in a new Extension Development Host window
```

## Inspiration

This extension draws inspiration from [vscode-cmake-tools](https://github.com/microsoft/vscode-cmake-tools) for UI/UX patterns and modern VS Code extension best practices.

## License

MIT License - see LICENSE file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes and version history.
