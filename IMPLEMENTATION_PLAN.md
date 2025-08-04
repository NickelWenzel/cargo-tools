# Implementation Plan: Cargo Tools Extension Manager Migration

## Overview

This document outlines the step-by-step migration from the current monolithic `extension.ts` to the new `CargoExtensionManager` architecture inspired by CMake Tools.

## Current State Analysis

The current `extension.ts` has these characteristics:
- Direct command registration in activate()
- Inline workspace initialization
- Manual component coordination
- No centralized error handling
- Limited logging and debugging support

## Migration Steps

### Phase 1: Foundation (Low Risk)

#### Step 1.1: Add Extension Manager (Parallel Implementation)
- ✅ Created `CargoExtensionManager` class
- ✅ Created new `extension-new.ts` as reference implementation
- ✅ Architecture roadmap documented

#### Step 1.2: Update Existing Components to Support Event System
The existing components need minimal changes to work with the extension manager:

```typescript
// cargoWorkspace.ts - Already has event emitters ✅
export class CargoWorkspace {
    readonly onDidChangeProfile = this._onDidChangeProfile.event;
    readonly onDidChangeTarget = this._onDidChangeTarget.event;
    readonly onDidChangeTargets = this._onDidChangeTargets.event;
    // ... existing implementation
}
```

The tree providers and status bar already accept CargoWorkspace in constructor ✅

#### Step 1.3: Command Registration Wrapper
Create a wrapper function for command registration with logging:

```typescript
// In cargoExtensionManager.ts ✅
private registerCommand<K extends keyof CargoExtensionManager>(name: K): vscode.Disposable {
    return vscode.commands.registerCommand(`cargo-tools.${name}`, async (...args: any[]) => {
        const correlationId = this.generateCorrelationId();
        try {
            console.log(`[${correlationId}] cargo-tools.${name} started`);
            // ... error handling and execution
        } catch (error) {
            console.error(`[${correlationId}] cargo-tools.${name} failed:`, error);
            throw error;
        }
    });
}
```

### Phase 2: Gradual Migration (Medium Risk)

#### Step 2.1: Switch Extension Entry Point
Replace the current `extension.ts` activate function:

```typescript
// Current extension.ts - backup as extension-old.ts
// New extension.ts uses CargoExtensionManager

export async function activate(context: vscode.ExtensionContext) {
    try {
        extensionManager = await CargoExtensionManager.create(context);
        return { getExtensionManager: () => extensionManager };
    } catch (error) {
        // Fallback to old implementation if needed
        console.error('Extension manager failed, using fallback');
        // ... fallback logic
    }
}
```

#### Step 2.2: Migrate Command Implementations
Move command implementations from extension.ts to extension manager:

```typescript
// Move these from extension.ts to CargoExtensionManager methods:
- build() ✅
- run() ✅  
- test() ✅
- debug() ✅
- clean() ✅
- selectProfile() ✅
- selectTarget() ✅
- refresh() ✅
```

#### Step 2.3: Enhanced Error Handling
Add comprehensive error handling and user feedback:

```typescript
// In command implementations ✅
try {
    // Command logic
} catch (error) {
    console.error(`Command failed:`, error);
    vscode.window.showErrorMessage(`Cargo Tools: ${name} failed - ${error}`);
    throw error;
}
```

### Phase 3: Enhanced Features (Higher Risk)

#### Step 3.1: Multi-Workspace Support
Add support for multiple workspace folders:

```typescript
export class CargoProjectController {
    private readonly folderToWorkspaceMap = new Map<vscode.WorkspaceFolder, CargoWorkspace>();
    
    async acknowledgeFolder(folder: vscode.WorkspaceFolder): Promise<CargoWorkspace[]> {
        // Create CargoWorkspace for each folder
        // Handle workspace-specific configurations
    }
}
```

#### Step 3.2: Enhanced Task Provider
Improve task provider with pseudoterminal support:

```typescript
export class CargoTaskTerminal implements vscode.Pseudoterminal {
    // Better progress reporting
    // Cancellation support
    // Real-time output streaming
}
```

#### Step 3.3: Configuration Management
Add reactive configuration management:

```typescript
export class CargoConfigurationReader {
    onChange<K extends keyof CargoConfiguration>(
        setting: K, 
        callback: (value: CargoConfiguration[K]) => any
    ): vscode.Disposable {
        // React to configuration changes
        // Update components automatically
    }
}
```

## Migration Validation

### Testing Strategy

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **Manual Testing**: Verify all commands work as expected
4. **Performance Testing**: Ensure no performance regression

### Rollback Plan

If issues arise:
1. Switch back to `extension-old.ts`
2. Disable new features via configuration
3. Gradual rollback of individual components

### Success Metrics

- All existing functionality preserved ✅
- Improved error handling and logging ✅
- Better separation of concerns ✅
- Foundation for future enhancements ✅

## Implementation Notes

### Current Status
- ✅ Extension Manager architecture defined
- ✅ Command registration wrapper implemented
- ✅ Event system integration planned
- ✅ Migration path documented

### Next Steps
1. Test extension manager with existing components
2. Create integration tests
3. Switch extension entry point
4. Migrate remaining commands
5. Add enhanced features incrementally

### Risk Mitigation
- Keep old extension.ts as backup
- Implement feature flags for new functionality
- Test thoroughly in development environment
- Gradual rollout to users

## Benefits Realized

### Immediate Benefits
- **Better Error Handling**: Centralized error handling with user feedback
- **Improved Logging**: Correlation IDs and structured logging
- **Code Organization**: Clear separation of concerns

### Future Benefits  
- **Multi-Workspace Support**: Foundation for multiple cargo workspaces
- **Enhanced Debugging**: Better integration with VS Code debugging
- **Extensibility**: Easier to add new commands and features
- **Maintainability**: Cleaner code structure and testing

## Conclusion

The Extension Manager architecture provides a solid foundation for the cargo-tools extension, following proven patterns from CMake Tools while maintaining compatibility with existing functionality. The phased migration approach minimizes risk while delivering immediate benefits in code organization and error handling.
