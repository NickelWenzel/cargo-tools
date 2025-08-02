# Cargo Tools Extension - Architecture Roadmap

Based on analysis of microsoft/vscode-cmake-tools, here are the key architectural improvements to implement:

## 1. Extension Manager Singleton Pattern

### Current State
- Commands are registered directly in `extension.ts`
- No centralized state management
- Limited coordination between components

### Proposed Changes
- Create `CargoExtensionManager` singleton class
- Centralize all command registration and coordination
- Manage workspace-level state and configuration
- Handle component lifecycle and disposal

```typescript
export class CargoExtensionManager implements vscode.Disposable {
    private static instance?: CargoExtensionManager;
    
    private constructor(private readonly extensionContext: vscode.ExtensionContext) {}
    
    static async create(context: vscode.ExtensionContext): Promise<CargoExtensionManager> {
        if (!CargoExtensionManager.instance) {
            CargoExtensionManager.instance = new CargoExtensionManager(context);
            await CargoExtensionManager.instance.init();
        }
        return CargoExtensionManager.instance;
    }
    
    async init() {
        // Initialize all components
        // Set up event subscriptions
        // Register commands
    }
}
```

## 2. Project Controller for Multi-Workspace Support

### Current State
- Single workspace assumption
- Limited workspace folder management

### Proposed Changes
- Create `CargoProjectController` class
- Support multiple workspace folders
- Manage per-folder cargo workspaces
- Handle workspace events (add/remove folders)

```typescript
export class CargoProjectController implements vscode.Disposable {
    private readonly folderToWorkspaceMap = new Map<vscode.WorkspaceFolder, CargoWorkspace>();
    
    async acknowledgeFolder(folder: vscode.WorkspaceFolder): Promise<CargoWorkspace[]> {
        // Discover cargo workspaces in folder
        // Create CargoWorkspace instances
        // Set up subscriptions
    }
}
```

## 3. Enhanced Command Registration Pattern

### Current State
- Direct command registration
- No error handling wrapper
- Limited logging

### Proposed Changes
- Centralized command registration with wrapper
- Automatic error handling and logging
- Command correlation IDs
- Progress reporting integration

```typescript
function registerCommand<K extends keyof CargoExtensionManager>(name: K) {
    return vscode.commands.registerCommand(`cargo-tools.${name}`, async (...args: any[]) => {
        const id = generateCorrelationId();
        try {
            log.debug(`[${id}] cargo-tools.${name} started`);
            const command = (extensionManager[name] as Function).bind(extensionManager);
            const result = await command(...args);
            log.debug(`[${id}] cargo-tools.${name} completed`);
            return result;
        } catch (error) {
            log.error(`[${id}] cargo-tools.${name} failed:`, error);
            throw error;
        }
    });
}
```

## 4. Robust Task Provider with Pseudoterminal

### Current State
- Basic task provider implementation
- Limited error handling in tasks
- No custom terminal integration

### Proposed Changes
- Implement `CargoTaskTerminal` extending `vscode.Pseudoterminal`
- Better progress reporting and cancellation
- Enhanced error handling and user feedback
- Support for background tasks (watch mode)

```typescript
export class CargoTaskTerminal implements vscode.Pseudoterminal {
    private writeEmitter = new vscode.EventEmitter<string>();
    private closeEmitter = new vscode.EventEmitter<number>();
    
    constructor(
        private command: string,
        private args: string[],
        private workspaceFolder: vscode.WorkspaceFolder,
        private options?: { cwd?: string; environment?: Environment }
    ) {}
    
    async open(): Promise<void> {
        // Execute cargo command with proper error handling
        // Stream output to terminal
        // Handle cancellation
    }
}
```

## 5. Event-Driven Component Communication

### Current State
- Direct component coupling
- Manual refresh calls
- Limited state synchronization

### Proposed Changes
- Implement event emitters for all major state changes
- Use subscription pattern for component communication
- Automatic UI updates on state changes

```typescript
export class CargoWorkspace {
    private readonly onTargetsChangedEmitter = new vscode.EventEmitter<CargoTarget[]>();
    private readonly onProfileChangedEmitter = new vscode.EventEmitter<CargoProfile>();
    
    get onTargetsChanged(): vscode.Event<CargoTarget[]> {
        return this.onTargetsChangedEmitter.event;
    }
    
    private notifyTargetsChanged(): void {
        this.onTargetsChangedEmitter.fire(this.targets);
    }
}
```

## 6. Configuration Management Improvements

### Current State
- Basic VS Code configuration reading
- No change event handling
- Limited workspace-specific settings

### Proposed Changes
- Create `CargoConfigurationReader` class
- Implement change event subscriptions
- Support workspace-folder-specific configuration
- Configuration validation and defaults

```typescript
export class CargoConfigurationReader implements vscode.Disposable {
    private readonly emitters = new Map<string, vscode.EventEmitter<any>>();
    
    onChange<K extends keyof CargoConfiguration>(
        setting: K, 
        callback: (value: CargoConfiguration[K]) => any
    ): vscode.Disposable {
        // Set up configuration change listener
        // Return disposable subscription
    }
}
```

## 7. Status Bar Integration Improvements

### Current State
- Basic status bar items
- Manual updates
- Limited responsiveness

### Proposed Changes
- Reactive status bar updates
- Better visual feedback for operations
- Context-aware button states
- Progress indication for long operations

```typescript
export class CargoStatusBar implements vscode.Disposable {
    private profileButton: StatusBarButton;
    private targetButton: StatusBarButton;
    private buildButton: StatusBarButton;
    
    setProfile(profile: CargoProfile): void {
        this.profileButton.text = profile;
        this.profileButton.update();
    }
    
    setIsBusy(busy: boolean): void {
        this.buildButton.isBusy = busy;
    }
}
```

## 8. Tree View Provider Enhancements

### Current State
- Basic tree providers
- Limited refresh logic
- No context menu integration

### Proposed Changes
- Event-driven tree refreshing
- Rich context menus with dynamic commands
- Drag-and-drop support where applicable
- Improved icons and theming

## Implementation Priority

1. **Phase 1: Core Architecture**
   - Extension Manager singleton
   - Command registration wrapper
   - Configuration reader
   - Event system foundation

2. **Phase 2: Enhanced Components**
   - Project Controller
   - Improved Task Provider
   - Status Bar enhancements
   - Tree view improvements

3. **Phase 3: Advanced Features**
   - Multi-workspace support
   - Background task management
   - Advanced debugging integration
   - Performance optimizations

## Benefits

1. **Maintainability**: Clear separation of concerns and standardized patterns
2. **Scalability**: Support for complex workspace configurations
3. **Reliability**: Better error handling and state management
4. **User Experience**: Responsive UI and better feedback
5. **Extensibility**: Easier to add new features and commands

## Migration Strategy

- Implement changes incrementally without breaking existing functionality
- Maintain backward compatibility for user configurations
- Add comprehensive tests for new architectural components
- Provide clear migration guide for any breaking changes
