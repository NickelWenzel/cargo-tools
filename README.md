# cargo-# Cargo Tools

A VS Code extension that provides comprehensive Rust development tools with build profiles, targets, and workspace integration. Inspired by the cmake-tools extension, this extension brings similar functionality to Rust projects.

## Features

### 🎯 Build Target Management
- Automatically discover binary, library, test, and example targets
- Easy target selection through tree view or quick pick
- Visual indicators for active targets
- Support for multi-target workspaces

### ⚙️ Build Profile Management  
- Switch between development and release profiles
- Visual profile selection in dedicated tree view
- Status bar integration showing current profile
- Custom profile arguments support

### 🏗️ Integrated Build System
- Build, run, test, debug, and clean commands
- Integrated terminal execution with proper working directory
- VS Code task integration with problem matchers
- Environment variable support

### 📁 Workspace Explorer
- Dedicated workspace tree view showing project structure
- Quick access to Cargo.toml and source files
- Target and dependency visualization
- Click-to-open file integration

### 📊 Status Bar Integration
- Current build profile indicator
- Active target display with appropriate icons
- Click-to-change functionality
- Clear visual feedback

### 🔧 Extensive Configuration
- Customizable cargo executable path
- Per-command argument configuration
- Environment variable settings
- Feature flags management
- Default profile selection

## Installation

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Cargo Tools"
4. Click Install

## Usage

### Getting Started

1. Open a Rust workspace containing a `Cargo.toml` file
2. The extension will automatically activate and scan your workspace
3. Use the Cargo Tools panel in the sidebar to view and manage your project

### Changing Build Profile

- Click the profile indicator in the status bar, or
- Use the Command Palette: `Cargo Tools: Select Build Profile`, or  
- Click on a profile in the Build Profiles tree view

### Selecting Build Target

- Click the target indicator in the status bar, or
- Use the Command Palette: `Cargo Tools: Select Build Target`, or
- Click on a target in the Build Targets tree view

### Running Commands

All commands are available through:
- Command Palette (`Ctrl+Shift+P`)
- Right-click context menus
- Dedicated buttons in tree views

Available commands:
- **Build**: Build the current target with current profile
- **Run**: Run the current executable target  
- **Test**: Run all tests
- **Debug**: Debug the current executable target
- **Clean**: Clean build artifacts
- **Refresh**: Refresh workspace information

## Configuration

Configure the extension through VS Code settings (`File > Preferences > Settings` and search for "Cargo Tools"):

```json
{
  "cargoTools.defaultProfile": "dev",
  "cargoTools.cargoPath": "cargo",
  "cargoTools.buildArgs": ["--verbose"],
  "cargoTools.runArgs": [],
  "cargoTools.testArgs": ["--", "--nocapture"],
  "cargoTools.environment": {
    "RUST_LOG": "debug"
  },
  "cargoTools.features": ["feature1", "feature2"],
  "cargoTools.allFeatures": false,
  "cargoTools.noDefaultFeatures": false
}
```

### Settings Reference

| Setting | Description | Default |
|---------|-------------|---------|
| `cargoTools.defaultProfile` | Default build profile to use | `"dev"` |
| `cargoTools.cargoPath` | Path to the cargo executable | `"cargo"` |
| `cargoTools.buildArgs` | Additional arguments for cargo build | `[]` |
| `cargoTools.runArgs` | Additional arguments for cargo run | `[]` |
| `cargoTools.testArgs` | Additional arguments for cargo test | `[]` |
| `cargoTools.environment` | Environment variables to set | `{}` |
| `cargoTools.features` | Default features to enable | `[]` |
| `cargoTools.allFeatures` | Enable all features by default | `false` |
| `cargoTools.noDefaultFeatures` | Disable default features | `false` |

## Integration with rust-analyzer

This extension is designed to work seamlessly with rust-analyzer:

- Environment variables set by Cargo Tools are available to rust-analyzer
- Build configurations remain consistent across tools
- Target selection affects both building and IDE features

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

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

## Requirements

- VS Code 1.102.0 or higher
- Rust and Cargo installed on your system
- A Rust workspace with a `Cargo.toml` file

## Known Issues

- TOML parsing is simplified and may not handle all edge cases
- Debug configuration assumes cppdbg (C++ debugger) for Rust debugging

## Roadmap

- [ ] Enhanced TOML parsing with proper library
- [ ] Support for custom build profiles beyond dev/release
- [ ] Integration with cargo workspaces (multi-package projects)
- [ ] Custom task templates
- [ ] Benchmark target support
- [ ] Cross-compilation target management
- [ ] Integration with cargo registry (crates.io)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the excellent [cmake-tools](https://github.com/microsoft/vscode-cmake-tools) extension
- Thanks to the Rust community for the amazing tooling ecosystem
- Built with the VS Code Extension API README

This is the README for your extension "cargo-tools". After writing up a brief description, we recommend including the following sections.

## Features

Describe specific features of your extension including screenshots of your extension in action. Image paths are relative to this README file.

For example if there is an image subfolder under your extension project workspace:

\!\[feature X\]\(images/feature-x.png\)

> Tip: Many popular extensions utilize animations. This is an excellent way to show off your extension! We recommend short, focused animations that are easy to follow.

## Requirements

If you have any requirements or dependencies, add a section describing those and how to install and configure them.

## Extension Settings

Include if your extension adds any VS Code settings through the `contributes.configuration` extension point.

For example:

This extension contributes the following settings:

* `myExtension.enable`: Enable/disable this extension.
* `myExtension.thing`: Set to `blah` to do something.

## Known Issues

Calling out known issues can help limit users opening duplicate issues against your extension.

## Release Notes

Users appreciate release notes as you update your extension.

### 1.0.0

Initial release of ...

### 1.0.1

Fixed issue #.

### 1.1.0

Added features X, Y, and Z.

---

## Following extension guidelines

Ensure that you've read through the extensions guidelines and follow the best practices for creating your extension.

* [Extension Guidelines](https://code.visualstudio.com/api/references/extension-guidelines)

## Working with Markdown

You can author your README using Visual Studio Code. Here are some useful editor keyboard shortcuts:

* Split the editor (`Cmd+\` on macOS or `Ctrl+\` on Windows and Linux).
* Toggle preview (`Shift+Cmd+V` on macOS or `Shift+Ctrl+V` on Windows and Linux).
* Press `Ctrl+Space` (Windows, Linux, macOS) to see a list of Markdown snippets.

## For more information

* [Visual Studio Code's Markdown Support](http://code.visualstudio.com/docs/languages/markdown)
* [Markdown Syntax Reference](https://help.github.com/articles/markdown-basics/)

**Enjoy!**
