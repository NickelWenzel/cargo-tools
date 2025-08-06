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
**Purpose:** Read-only project structure overview with selection state mirroring

#### Content Structure
- **Root Node:** Project name (from Cargo.toml name field)
- **Package Organization:** Same workspace member grouping as current tree view
- **Target Categorization:** Same target type categorization (bins, examples, benchmarks, etc.)
- **Features Integration:**
  - **Root Level:** "Features" node containing "All features" item
  - **Package Level:** Each package has its own "Features" node with package-specific features

#### Selection State Mirroring
The Project Outline Pane reflects the current selections from the Project Status Pane:
- **Package Selection:** Visually indicated using package emoji (📦) on the right side of labels
- **Build Target Selection:** Show which build target is currently selected using hammer emoji (🔨)
  - Handles special case: when "lib" is selected as build target, only library targets in the selected package show the indicator
- **Run Target Selection:** Show which run target is currently selected using rocket emoji (🚀)
- **Benchmark Target Selection:** Show which benchmark target is currently selected using lightning emoji (⚡)
- **Feature Selection:** Show which features are currently selected (both at root and package level)

#### Visual Indicators
- **Package Selection:** Uses package emoji (📦) on the right side of selected package labels
- **Build Target Selection:** Uses hammer emoji (🔨) on the right side of selected build target labels
- **Run Target Selection:** Uses rocket emoji (🚀) on the right side of selected run target labels
- **Benchmark Target Selection:** Uses lightning emoji (⚡) on the right side of selected benchmark target labels
- **Default Targets:** Uses `star` icon with highlight color
- **Feature Selection:** Uses checkmark icons for selected features
- **State Synchronization:** Real-time updates when selections change in Project Status Pane
- **Icon Reference:** Label emoji pattern follows microsoft/vscode-cmake-tools visual patterns
- **Hierarchical Context:** Clear visual hierarchy showing project → packages → targets/features

#### Root Features Node Behavior
- **Content:** Only shows "All features" item regardless of package selection
- **Purpose:** Provides workspace-level feature selection independent of package context
- **Consistency:** Maintains stable content while package-level features reflect specific selections

#### Context Menu Actions
The Project Outline Pane provides comprehensive context menus for direct selection control:

##### Package Context Menu
- **Select Package:** Sets the package as the selected package in Project Status
- **Unselect Package:** Sets package selection to "All" in Project Status
- **Availability:** Available on all workspace member nodes

##### Target Context Menu
- **Set as Build Target:** 
  - Available on: lib, bin, example, bench targets
  - Effect: Sets target as selected build target and switches to target's package
  - Special handling: Library targets set "lib" as build target selection
- **Set as Run Target:**
  - Available on: bin, example targets only
  - Effect: Sets target as selected run target and switches to target's package
- **Set as Benchmark Target:**
  - Available on: bench targets only
  - Effect: Sets target as selected benchmark target and switches to target's package
- **Unset Actions:** All target types include corresponding "Unset" actions to clear selection

##### Feature Interaction
- **Checkbox Behavior:** Features act as toggleable checkboxes when their package is selected
- **Click to Toggle:** Direct click toggles feature selection on/off
- **Conditional Availability:** Only interactive when the feature's package is currently selected
- **Real-time Sync:** Changes immediately reflect in Project Status pane feature selection

#### Interactions
- **Context Menus:** Interactive context menus for selection control
  - **Package Nodes:** Select/Unselect package (mirrors to Project Status)
  - **Build Targets:** Set/Unset as build target (includes lib, bin, example, bench types)
  - **Run Targets:** Set/Unset as run target (bin and example types only)
  - **Benchmark Targets:** Set/Unset as benchmark target (bench types only)
  - **Auto Package Selection:** Target selection automatically switches to the target's package
- **Feature Checkboxes:** Interactive feature selection
  - **Condition:** Only when the feature's package is currently selected
  - **Behavior:** Click to toggle feature on/off
  - **Synchronization:** Real-time mirroring to Project Status pane feature selection
- **Tree Navigation:** Tree expansion/collapse for structural organization
- **Read-Only Structure:** Project structure display cannot be modified, only selections change
- **Goal:** Comprehensive project overview with direct selection control capabilities

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
