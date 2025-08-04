# Target-Specific Commands and Context Menu

## Overview
The BUILD TARGETS tree view now supports target-specific commands through context menus, following the CMake Tools extension pattern for individual target operations.

## Features

### Context Menu Commands
Right-click on any target in the BUILD TARGETS tree to access:

#### For All Targets
- **Build Target**: Build the specific target with proper workspace awareness
- **Set as Default Target**: Make this target the default for general build/run commands

#### For Executable Targets (bin, example)
- **Run Target**: Execute the target directly
- **Debug Target**: Start a debugging session for the target

#### For Test Targets
- **Test Target**: Run the specific test target

### Workspace Support
All target-specific commands properly handle Cargo workspaces:
- Commands automatically include `-p <package-name>` for workspace members
- Working directory is set to the workspace root
- Environment variables and configuration are applied correctly

### Command Execution
Commands execute in VS Code's integrated terminal, providing:
- Real-time output visibility
- Proper environment variable inheritance
- Integration with VS Code's terminal management

## Implementation Details

### Context Menu Integration
Following CMake Tools patterns:
```json
{
  "command": "cargo-tools.buildTarget",
  "when": "view == cargoToolsTargets && viewItem =~ /cargoTarget/",
  "group": "target@1"
}
```

### Target-Specific Command Arguments
Commands construct appropriate cargo arguments:
- **Binary targets**: `cargo build --bin <target-name>`
- **Library targets**: `cargo build --lib`
- **Example targets**: `cargo run --example <target-name>`
- **Test targets**: `cargo test --test <target-name>`
- **Workspace members**: `cargo build -p <package-name> --bin <target-name>`

### Command Structure Example
For a binary target `cli-tool` in package `cli`:
```bash
cargo build --release -p cli --bin cli-tool
```

### Debug Integration
Debug targets:
1. Build the target first: `cargo build -p <package> --bin <target>`
2. Launch VS Code debugger with proper executable path
3. Use workspace root as working directory

## Benefits

### User Experience
- **Intuitive workflow**: Right-click → select action, just like CMake Tools
- **Visual feedback**: Commands show in terminal with real-time output
- **Smart defaults**: Commands adapt to target type automatically

### Developer Efficiency
- **No manual command construction**: Extension handles cargo arguments
- **Workspace awareness**: Automatically handles multi-package workspaces  
- **Integrated debugging**: One-click debug setup for executable targets

### Extensibility
- **Consistent patterns**: Following CMake Tools for future enhancements
- **Configuration support**: Respects all cargoTools configuration options
- **Terminal integration**: Leverages VS Code's terminal for familiar UX

## Configuration

### Relevant Settings
All existing `cargoTools` settings apply to target-specific commands:
- `cargoTools.cargoPath`: Path to cargo executable
- `cargoTools.buildArgs`: Additional build arguments
- `cargoTools.runArgs`: Additional run arguments  
- `cargoTools.testArgs`: Additional test arguments
- `cargoTools.environment`: Environment variables
- `cargoTools.features`: Feature flags to enable

### Example Configuration
```json
{
  "cargoTools.environment": {
    "RUST_LOG": "debug"
  },
  "cargoTools.buildArgs": ["--verbose"],
  "cargoTools.features": ["serde"]
}
```

## Usage Examples

### Building a Specific Target
1. Navigate to BUILD TARGETS tree view
2. Expand workspace member (if applicable)
3. Expand target type (bin, lib, etc.)
4. Right-click on target → "Build Target"

### Running an Example
1. Find example target in BUILD TARGETS
2. Right-click → "Run Target"
3. Terminal opens with: `cargo run --example <name>`

### Debugging a Binary
1. Right-click binary target → "Debug Target"
2. Target builds automatically
3. Debugger launches with proper configuration

### Setting Default Target
1. Right-click any target → "Set as Default Target"
2. Target becomes default for `cargo-tools.build` and `cargo-tools.run` commands
3. Status bar updates to show new default target
