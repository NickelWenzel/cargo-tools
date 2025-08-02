# Workspace Member Grouping Feature

## Overview
The BUILD TARGETS tree view now supports grouping targets by workspace member when working with Cargo workspaces, following the CMake Tools extension pattern for organizing targets by project.

## Behavior

### Multi-Member Workspaces
When `cargoTools.groupTargetsByWorkspaceMember` is enabled (default) and the workspace contains multiple members:
```
BUILD TARGETS
├── core
│   ├── lib (1)
│   │   └── core (lib)
├── cli  
│   ├── bin (2)
│   │   ├── cli-main (bin)
│   │   └── cli-tool (bin)
├── web-server
│   ├── bin (2)
│   │   ├── server (bin)
│   │   └── simple-server (bin)
│   ├── lib (1)
│   │   └── web_server (lib)
├── utils
│   ├── lib (1)
│   │   └── utils (lib)
│   ├── example (2)
│   │   ├── string-utils (example)
│   │   └── performance (example)
```

### Single Package or Disabled Grouping
When the workspace contains only one package or grouping is disabled:
```
BUILD TARGETS
├── bin (4)
│   ├── cli-main (bin)
│   ├── cli-tool (bin)
│   ├── server (bin)
│   └── simple-server (bin)
├── lib (3)
│   ├── core (lib)
│   ├── web_server (lib)
│   └── utils (lib)
├── example (2)
│   ├── string-utils (example)
│   └── performance (example)
```

## Configuration

### cargoTools.groupTargetsByWorkspaceMember
- **Type:** `boolean`
- **Default:** `true`
- **Description:** Group targets by workspace member in the BUILD TARGETS tree view. When enabled, targets are first organized by workspace member, then by type (bin, lib, etc.). When disabled, targets are grouped directly by type.

## Implementation Details

### CMake Tools Patterns Applied
- **Workspace-aware organization:** Similar to how CMake Tools groups targets by project in multi-project workspaces
- **Configuration-driven UI:** User can control grouping behavior via settings
- **Automatic detection:** Automatically detects multi-member workspaces using `cargo metadata`
- **Fallback support:** Gracefully falls back to type-based grouping for single packages

### Tree Structure Classes
- `WorkspaceMemberItem`: Represents a workspace member node with package name and targets
- `TargetGroupItem`: Represents a target type group (bin, lib, example, etc.)
- `TargetTreeItem`: Represents individual targets

### Workspace Detection
The extension uses `cargo metadata` to detect workspace members and their packages:
- Parses `workspace_members` field from metadata
- Associates targets with their package names
- Falls back to manual discovery when metadata is unavailable

## Benefits
1. **Better Organization:** Large workspaces with many members are easier to navigate
2. **Clear Ownership:** Targets are clearly associated with their workspace members
3. **Flexible UI:** Users can choose between workspace-based or type-based organization
4. **CMake Tools Compatibility:** Familiar patterns for users coming from C/C++ development
