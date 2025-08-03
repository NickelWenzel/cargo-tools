# Cargo Tools Extension - Feature Specification

## Overview
This document outlines the feature specification for the Cargo Tools extension UI redesign, following patterns from the microsoft/vscode-cmake-tools extension for consistency and usability.

## Tree Panes Architecture

The extension will have three main tree panes in the extension sidebar:

### 1. Project Status Pane
**Purpose:** Configuration and status display with actionable controls

#### Build Profile Selection
- **Display:** Current build profile (dev/release)
- **Interaction:** Click to select different profile
- **Implementation:** Dropdown/quick pick interface

#### Package Selection
- **Single Package/Non-Workspace:**
  - Display: "default" (read-only, no selection needed)
  - No interaction required
- **Multi-Package Workspace:**
  - Display: List of packages with "All" option at top
  - "All" corresponds to no package specification in cargo commands
  - Individual packages listed below
  - **Command Line Impact:** Selected package adds `-p <package>` to cargo commands

#### Target Selection

##### Build Target Selection
- **When "All" Package Selected:**
  - Display: Only "All" option available
  - No specific target specification in commands
- **When Specific Package Selected:**
  - Display: Categorized targets from selected package:
    - "All" (no target specification)
    - "lib" (if library exists)
    - "bins" group (all binary targets)
    - "examples" group (all example targets) 
    - "benchmarks" group (all benchmark targets)
  - **Command Line Impact:** 
    - lib: `--lib`
    - bin: `--bin <target_name>`
    - example: `--example <target_name>`
    - benchmark: `--bench <target_name>`
- **Button that will trigger 'cargo build' command:**
  - Should respect target selection as described above
- **Implementation:** Dropdown/quick pick interface

##### Run Target Selection
- **When "All" Package Selected:**
  - Disabled
- **When Specific Package Selected:**
  - Display "bins" and "examples" from selected package:
  - **Command Line Impact:** 
    - bin: `--bin <target_name>`
    - example: `--example <target_name>`
- **Button that will trigger 'cargo run' command:**
  - Should respect target selection as described above
- **Implementation:** Dropdown/quick pick interface

##### Benchmark Target Selection
- **When "All" Package Selected:**
  - Display: Only "All" option available
  - No specific target specification in commands
- **When Specific Package Selected:**
  - Display "benchmarks" from selected package
  - **Command Line Impact:** 
    - benchmark: `--bench <target_name>`
- **Button that will trigger 'cargo build' command:**
  - Should respect target selection as described above
- **Implementation:** Dropdown/quick pick interface

##### Feature Selection
- A list of selectable items
  - Implemented as a checkbox
- Always has item "all features" which is the default selection
- **When "All" Package Selected:**
  - Don't show any other item
- **When Specific Package Selected:**
  - Show the features of the selected package as selectable items as well
- This should also react to changes in the package selection


### 2. Project Outline Pane
**Purpose:** Read-only project structure overview

#### Content
- **Based On:** Current "Build Targets" tree view
- **Functionality:** 
  - Same tree structure and organization
  - Same workspace member grouping
  - Same target type categorization
- **Interactions:** 
  - **Removed:** All context menu actions
  - **Retained:** Tree expansion/collapse only
- **Goal:** Pure informational view of project structure

### 3. Pinned Commands Pane
**Purpose:** Quick access to frequently used commands (future implementation)

#### Current State
- **Display:** Empty pane
- **Placeholder:** "No pinned commands" or similar
- **Future:** Will contain user-customizable quick access commands

## Implementation Guidelines

### Reference Implementation
- **Primary Reference:** microsoft/vscode-cmake-tools extension
- **UI Patterns:** Follow CMake Tools patterns for:
  - Tree view organization
  - Status display
  - Configuration selection
  - Action button placement
  - User interaction flows

### Technical Architecture
- **State Management:** Centralized configuration state
- **Event Handling:** Reactive updates between panes
- **Command Generation:** Dynamic based on current selections
- **Persistence:** Save user selections across sessions

### User Experience Goals
1. **Clarity:** Clear indication of current configuration
2. **Efficiency:** Quick access to common build operations
3. **Consistency:** Familiar patterns from CMake Tools
4. **Flexibility:** Support both simple and complex project structures
5. **Discoverability:** Intuitive organization and labeling

## Command Line Generation Logic

### Base Command Pattern
```
cargo build [--release] [-p <package>] [--lib|--bin <name>|--example <name>|--bench <name>]
```

### Selection Matrix
| Profile | Package | Target | Generated Command |
|---------|---------|--------|-------------------|
| dev     | All     | All    | `cargo build` |
| release | All     | All    | `cargo build --release` |
| dev     | pkg1    | All    | `cargo build -p pkg1` |
| dev     | pkg1    | lib    | `cargo build -p pkg1 --lib` |
| dev     | pkg1    | bin:foo| `cargo build -p pkg1 --bin foo` |
| release | pkg1    | bin:foo| `cargo build --release -p pkg1 --bin foo` |

## Migration Strategy

### Phase 1: Core Infrastructure
1. Create new tree providers for the three panes
2. Implement configuration state management
3. Set up reactive update system

### Phase 2: Project Status Implementation
1. Build profile selection UI
2. Package selection logic and UI
3. Build target selection with dependency on package
4. Action button integration

### Phase 3: Project Outline Migration
1. Copy current Build Targets tree structure
2. Remove context menu actions
3. Make read-only

### Phase 4: Polish and Integration
1. Styling and icon consistency
2. State persistence
3. Error handling and edge cases
4. Testing and validation

## Future Enhancements

### Pinned Commands Pane
- User-customizable command shortcuts
- Predefined common command templates
- Command history and favorites

### Advanced Features
- Multiple configuration profiles
- Custom command templates
- Integration with VS Code tasks
- Build status indicators
- Error highlighting and navigation

---

**Reference:** This specification follows patterns established in microsoft/vscode-cmake-tools for consistency with existing VS Code extension UX patterns.
