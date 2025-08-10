# Cargo Tools Command Substitution

The Cargo Tools extension provides command substitution support that allows you to use dynamic values from the current Cargo Tools state in VS Code settings and configurations. This is particularly useful for configuring Rust Analyzer and other tools that need to know about your current project state.

## Available Commands

The following VS Code commands are available for use in settings and configuration files:

### Core Selection Commands

- `cargo-tools.selectedPackage` - Returns the currently selected package name
- `cargo-tools.selectedProfile` - Returns the currently selected build profile (e.g., "dev", "release")
- `cargo-tools.selectedTargets` - Returns a comma-separated list of currently selected targets
- `cargo-tools.selectedFeatures` - Returns a comma-separated list of currently selected features

### Cargo Command Generation

- `cargo-tools.cargoArgs` - Returns the complete cargo arguments for the current selection
- `cargo-tools.cargoCommand` - Returns the complete cargo command with all arguments

## Usage Examples

### Rust Analyzer Configuration

You can use these commands in your VS Code settings to dynamically configure Rust Analyzer based on your current Cargo Tools selections:

```json
{
  "rust-analyzer.cargo.buildScripts.overrideCommand": [
    "cargo",
    "check",
    "--quiet",
    "--message-format=json",
    "--manifest-path",
    "Cargo.toml",
    "--profile",
    "${command:cargo-tools.selectedProfile}",
    "--package", 
    "${command:cargo-tools.selectedPackage}"
  ],
  "rust-analyzer.cargo.target": "${command:cargo-tools.selectedTargets}",
  "rust-analyzer.cargo.features": "${command:cargo-tools.selectedFeatures}"
}
```

### Task Configuration

You can also use these commands in VS Code tasks:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Current Package Test",
      "type": "shell",
      "command": "cargo",
      "args": [
        "test",
        "--package",
        "${command:cargo-tools.selectedPackage}",
        "--profile",
        "${command:cargo-tools.selectedProfile}"
      ],
      "group": "test"
    }
  ]
}
```

### Launch Configuration

Use with VS Code launch configurations for debugging:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Current Package",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/${command:cargo-tools.selectedPackage}",
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## How It Works

The command substitution system monitors the current state of Cargo Tools selections and provides that information through VS Code commands. When VS Code encounters a `${command:cargo-tools.*}` substitution in your configuration, it calls the corresponding command to get the current value.

This approach is modeled after the CMake Tools extension and provides a reliable way to keep your tool configurations in sync with your current project state.

## Notes

- Commands return empty strings or sensible defaults when no selection is made
- The system automatically updates when you change selections in Cargo Tools
- All commands are synchronous and return immediately with the current state
