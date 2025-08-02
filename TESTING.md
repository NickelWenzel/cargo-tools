# Cargo Tools Extension - Testing Guide

## Quick Start

The Cargo Tools extension has been successfully created and is ready for testing!

## What's Been Built

### ✅ Core Features Implemented

1. **Extension Structure**
   - TypeScript-based VS Code extension
   - Webpack bundling for production builds
   - Proper activation events for Rust workspaces

2. **Cargo Workspace Management**
   - Automatic detection of `Cargo.toml` files
   - Target discovery (binary, library, test, etc.)
   - Build profile management (dev/release)
   - Cargo command execution

3. **User Interface**
   - **Build Profiles Tree View**: Select between dev and release profiles
   - **Build Targets Tree View**: Select and manage build targets
   - **Workspace Tree View**: Explore project structure and files
   - **Status Bar**: Shows current profile and target with click-to-change
   - **Commands**: Build, run, test, debug, clean, refresh

4. **VS Code Integration**
   - Task provider for cargo commands
   - Problem matchers for error detection
   - Command palette integration
   - Settings configuration
   - Context-aware command availability

## Testing the Extension

### Method 1: Extension Development Host (Recommended)

1. **Open the Extension Project**
   ```bash
   cd /home/nickel/Programming/repos/cargo_tools
   code .
   ```

2. **Start Extension Development Host**
   - Press `F5` or go to `Run and Debug` view
   - Select "Run Extension" and press play
   - This opens a new VS Code window with the extension loaded

3. **Open Test Rust Project**
   - In the new window, open the test project:
     ```
     File → Open Folder → /home/nickel/Programming/repos/cargo_tools/test-rust-project
     ```

4. **Test Extension Features**
   - Look for the "Cargo Tools" panel in the sidebar
   - Check status bar for profile/target indicators
   - Try the commands from Command Palette (`Ctrl+Shift+P`)

### Method 2: Package and Install

1. **Install vsce (if not installed)**
   ```bash
   npm install -g vsce
   ```

2. **Package the Extension**
   ```bash
   vsce package
   ```

3. **Install the .vsix File**
   ```bash
   code --install-extension cargo-tools-0.0.1.vsix
   ```

## Extension Features to Test

### 1. Tree Views
- **Build Profiles Panel**: Should show "Development" and "Release" options
- **Build Targets Panel**: Should show discovered targets (main, helper, testlib)
- **Workspace Panel**: Should show project structure and files

### 2. Status Bar
- Look for profile indicator (e.g., "🔧 Development")
- Look for target indicator (e.g., "▶️ main")
- Click on them to change profile/target

### 3. Commands (via Command Palette)
- `Cargo Tools: Build` - Build current target
- `Cargo Tools: Run` - Run executable target
- `Cargo Tools: Test` - Run tests
- `Cargo Tools: Debug` - Debug executable target
- `Cargo Tools: Clean` - Clean build artifacts
- `Cargo Tools: Select Build Profile` - Change profile
- `Cargo Tools: Select Build Target` - Change target
- `Cargo Tools: Refresh` - Refresh workspace

### 4. Configuration
- Open Settings (`Ctrl+,`)
- Search for "Cargo Tools"
- Verify configuration options are available

## Expected Behavior

### When Opening a Rust Project
1. Extension activates automatically
2. "Cargo Tools" appears in sidebar
3. Status bar shows current profile and target
4. Tree views populate with discovered information

### When Using Commands
1. **Build**: Opens terminal and runs `cargo build` with appropriate flags
2. **Run**: Only available for executable targets
3. **Test**: Runs `cargo test` with configured arguments
4. **Profile/Target Selection**: Updates UI and subsequent commands

## Test Project Structure

The included test project has:
- **Multiple targets**: main binary, helper binary, library
- **Dependencies**: serde with features
- **Features**: custom feature flags for testing
- **Tests**: unit tests in lib.rs

## Troubleshooting

### Extension Not Activating
- Ensure the folder contains a `Cargo.toml` file
- Check VS Code's Developer Console for errors (`Help → Toggle Developer Tools`)

### Commands Not Working
- Verify cargo is installed and in PATH: `cargo --version`
- Check extension configuration for custom cargo path

### Tree Views Empty
- Try the "Refresh" command
- Check that `cargo metadata` works in the project directory

## Development Notes

### Architecture Highlights
- **Modular Design**: Separate classes for workspace, targets, profiles, and UI components
- **Event-Driven**: Uses VS Code's event system for UI updates
- **Configuration**: Extensive settings for customization
- **Error Handling**: Graceful fallbacks for missing tools or invalid projects

### Similar to cmake-tools
- Tree-based project exploration
- Profile and target selection
- Status bar integration
- Task provider system
- Command integration

### Integration Points
- Works with rust-analyzer for consistent environments
- Integrates with VS Code tasks and problem matchers
- Respects workspace configuration
- Supports multi-root workspaces

## Next Steps

1. **Test with Real Projects**: Try the extension with existing Rust projects
2. **Feature Additions**: Add workspace support, custom profiles, cross-compilation
3. **Polish**: Improve error messages, add more configuration options
4. **Performance**: Optimize for large workspaces
5. **Publishing**: Prepare for VS Code marketplace publication

The extension provides a solid foundation for Rust development in VS Code with cmake-tools-like functionality!
