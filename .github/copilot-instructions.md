# Cargo Tools Extension

This VS Code extension provides Rust development tools with build profiles, targets, and workspace integration, similar to the cmake-tools extension.

## Architecture

### Core Classes

- **CargoWorkspace**: Main class that manages the Rust workspace, parses Cargo.toml, discovers targets, and executes cargo commands
- **CargoTarget**: Represents a build target (binary, library, test, etc.)
- **CargoProfile**: Enum for build profiles (dev, release)

### UI Components

- **ProfilesTreeProvider**: Tree view showing available build profiles
- **TargetsTreeProvider**: Tree view showing available build targets  
- **WorkspaceTreeProvider**: Tree view showing workspace structure and files
- **StatusBarProvider**: Status bar items showing current profile and target
- **CargoTaskProvider**: Provides VS Code tasks for cargo commands

### Key Features

1. **Build Profile Selection**: Switch between dev and release profiles
2. **Target Selection**: Select which binary/library to build/run
3. **Integrated Commands**: Build, run, test, debug, clean commands
4. **Tree Views**: Organized panels for profiles, targets, and workspace
5. **Status Bar**: Quick access to current configuration
6. **Task Integration**: VS Code tasks for all cargo operations
7. **Configuration**: Extensive settings for customization

### Extension Integration

The extension integrates with rust-analyzer by:
- Setting environment variables for current configuration
- Providing consistent build settings across tools
- Exposing current target/profile state

### Development Guidelines

- Follow VS Code extension best practices
- Use TypeScript for type safety
- Implement proper error handling
- Provide clear user feedback
- Support workspace-level configuration
- Maintain backward compatibility

### Commands

- `cargo-tools.build`: Build current target with current profile
- `cargo-tools.run`: Run current executable target
- `cargo-tools.test`: Run tests
- `cargo-tools.debug`: Debug current executable target
- `cargo-tools.clean`: Clean build artifacts
- `cargo-tools.selectProfile`: Select build profile
- `cargo-tools.selectTarget`: Select build target
- `cargo-tools.refresh`: Refresh workspace information

### Configuration

All settings are under the `cargoTools` namespace:
- `defaultProfile`: Default build profile
- `cargoPath`: Path to cargo executable
- `buildArgs`: Additional build arguments
- `runArgs`: Additional run arguments
- `testArgs`: Additional test arguments
- `environment`: Environment variables
- `features`: Default features to enable
- `allFeatures`: Enable all features
- `noDefaultFeatures`: Disable default features
