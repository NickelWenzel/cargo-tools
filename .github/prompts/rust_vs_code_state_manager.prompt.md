---
mode: agent
model: Claude Sonnet 4.5 (copilot)
tools: ['search', 'edit', 'fetch', 'terminal']
description: 'Implement VS Code State Manager with Subscription Support'
---

# Task: Implement VS Code State Manager with Subscription Support

## Objective

Complete the implementation of `VSCodeStateManager` in `packages/cargo_tools_vscode/src/vs_code_state_manager.rs` by:
1. Refactoring the VS Code API interface from free functions to a structured class-based approach
2. Implementing the subscription mechanism for state change notifications
3. Ensuring proper cleanup of subscriptions via `reset_subscriptions()`

## Context

The `StateManager` trait is defined in `packages/cargo_tools/src/state_manager.rs`:

```rust
#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;

    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue>(&self, value: T) -> Result<(), Self::UpdateError>;

    fn subscribe(&self, on_change: impl AsyncFn(&State));
    fn reset_subscriptions();
}
```

Currently, `VSCodeStateManager` has stub implementations for `subscribe()` and `reset_subscriptions()`. The VS Code API in `packages/cargo_tools_vscode/src/vs_code_api.rs` uses free functions imported from TypeScript via `wasm_bindgen`.

## Requirements

### 1. Refactor VS Code API to Class-Based Structure

**Current Structure (Free Functions):**
```rust
#[wasm_bindgen(raw_module = "../stateManager.ts")]
extern "C" {
    pub fn get_state(key: &str) -> Option<JsValue>;
    pub async fn update_state(key: String, value: JsValue) -> Result<(), JsValue>;
}
```

**New Structure (Class-Based):**
```rust
#[wasm_bindgen(raw_module = "../stateManager.ts")]
extern "C" {
    pub type StateManagerApi;
    
    #[wasm_bindgen(constructor)]
    pub fn new() -> StateManagerApi;
    
    #[wasm_bindgen(method)]
    pub fn get_state(this: &StateManagerApi, key: &str) -> Option<JsValue>;
    
    #[wasm_bindgen(method, catch)]
    pub async fn update_state(
        this: &StateManagerApi,
        key: String,
        value: JsValue,
    ) -> Result<(), JsValue>;
    
    #[wasm_bindgen(method)]
    pub fn on_state_changed(
        this: &StateManagerApi,
        callback: &js_sys::Function,
    );
    
    #[wasm_bindgen(method)]
    pub fn fire_state_changed(this: &StateManagerApi, state: JsValue);
    
    #[wasm_bindgen(method)]
    pub fn reset_subscriptions(this: &StateManagerApi);
}
```

### 2. Update VSCodeStateManager Structure

The Rust struct should maintain the current state, compare values on updates, and store closures to keep them alive:

```rust
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;

pub struct VSCodeStateManager {
    api: StateManagerApi,
    current_state: RefCell<State>,
    // Store closures to keep them alive - they must not be dropped
    callbacks: RefCell<Vec<Closure<dyn FnMut(JsValue)>>>,
}

impl VSCodeStateManager {
    pub fn new() -> Self {
        // Initialize with default state or load from VS Code storage
        let initial_state = Self::load_initial_state();
        
        Self {
            api: StateManagerApi::new(),
            current_state: RefCell::new(initial_state),
            callbacks: RefCell::new(Vec::new()),
        }
    }
    
    fn load_initial_state() -> State {
        // Load each state value from storage or use defaults
        State {
            selected_package: SelectedPackage::default(),
            selected_build_target: SelectedBuildTarget::default(),
            // ... other fields
        }
    }
}
```

**Important:** The `callbacks` vector stores all registered closures. This is critical because:
- Closures passed to JavaScript must not be dropped while JavaScript might call them
- Using `forget()` causes memory leaks
- Storing closures in a vector keeps them alive and allows cleanup via `reset_subscriptions()`

### 3. Implement State Change Subscriptions

The `subscribe()` method should register callbacks with the TypeScript event emitter and store the closure:

**Expected Implementation Pattern:**
```rust
fn subscribe(&self, on_change: impl AsyncFn(&State)) {
    // Create a Closure that wraps the async callback
    let closure = Closure::wrap(Box::new(move |js_state: JsValue| {
        // Deserialize State from JsValue
        if let Ok(state) = from_value::<State>(js_state) {
            // Call the async handler
            on_change(&state);
        }
    }) as Box<dyn FnMut(JsValue)>);
    
    // Register with the event emitter
    self.api.on_state_changed(closure.as_ref().unchecked_ref());
    
    // Store the closure to keep it alive - DO NOT use forget()!
    // The closure will be dropped when reset_subscriptions() is called
    self.callbacks.borrow_mut().push(closure);
}
```

**Critical:** The closure MUST be stored in `self.callbacks` to keep it alive. Do not use `Closure::forget()` as it causes memory leaks. When `reset_subscriptions()` is called, clearing the vector will properly drop all closures.

### 4. Implement Update with Change Detection

The `update()` method should:
- Update the persistent storage via the API
- Compare the new state with the current state
- Update the current state
- Fire the state changed event if there are actual changes

```rust
async fn update<T: StateValue>(&self, value: T) -> Result<(), Self::UpdateError> {
    // Serialize and store in VS Code storage
    let js_value = to_value(&value).map_err(StateManagerError::SerializationError)?;
    self.api
        .update_state(T::KEY.to_string(), js_value)
        .await
        .map_err(|e| StateManagerError::UpdateError(e.as_error_string()))?;
    
    // Update the current state and check if it changed
    let mut current_state = self.current_state.borrow_mut();
    let old_value = self.get_value_from_state::<T>(&current_state);
    
    // Update the state field
    self.set_value_in_state::<T>(&mut current_state, value);
    
    // If the value changed, fire the event
    if old_value != self.get_value_from_state::<T>(&current_state) {
        drop(current_state); // Release the borrow before calling fire_state_changed
        let state_copy = self.current_state.borrow().clone();
        let js_state = to_value(&state_copy).map_err(StateManagerError::SerializationError)?;
        self.api.fire_state_changed(js_state);
    }
    
    Ok(())
}
```

### 5. Implement Subscription Cleanup

```rust
fn reset_subscriptions() {
    // Clear all event listeners in TypeScript
    self.api.reset_subscriptions();
    
    // Drop all stored closures - this properly cleans them up
    self.callbacks.borrow_mut().clear();
}
```

**Note:** Clearing the `callbacks` vector will drop all closures, which properly releases their resources. This is the correct way to clean up closures in wasm-bindgen, not using `forget()`.

### 6. Update TypeScript Side (stateManager.ts)

The TypeScript `StateManagerApi` class should:
- Manage VS Code `Memento` for persistent state
- Use an EventEmitter for state change notifications
- Provide methods to register callbacks and fire events

**Example Structure:**
```typescript
import * as vscode from 'vscode';
import { EventEmitter } from 'events';

export class StateManagerApi {
    private context: vscode.ExtensionContext;
    private stateChangedEmitter: EventEmitter;

    constructor() {
        // Get the extension context (needs to be set externally or passed in)
        this.context = getExtensionContext();
        this.stateChangedEmitter = new EventEmitter();
    }

    get_state(key: string): any {
        return this.context.workspaceState.get(key);
    }

    async update_state(key: string, value: any): Promise<void> {
        await this.context.workspaceState.update(key, value);
    }

    on_state_changed(callback: (state: any) => void): void {
        this.stateChangedEmitter.on('changed', callback);
    }

    fire_state_changed(state: any): void {
        this.stateChangedEmitter.emit('changed', state);
    }

    reset_subscriptions(): void {
        this.stateChangedEmitter.removeAllListeners('changed');
    }
}
```

**Notes:**
- The TypeScript class uses Node.js `EventEmitter` for event handling
- Callbacks are registered via `on_state_changed` and stored by the emitter
- `fire_state_changed` is called from Rust when state actually changes
- `reset_subscriptions` clears all event listeners

## Implementation Steps

### Step 1: Update vs_code_api.rs

1. Replace free function declarations with class-based `StateManagerApi`
2. Add subscription-related methods
3. Keep backward compatibility for other VS Code APIs (`execute_command`, `log`, etc.)

### Step 2: Update TypeScript stateManager.ts

1. Create `StateManagerApi` class
2. Implement state persistence using `vscode.ExtensionContext`
3. Implement subscription management
4. Implement change notification system

### Step 3: Update vs_code_state_manager.rs

1. Add `api`, `current_state`, and `callbacks` fields to `VSCodeStateManager`
2. Update `new()` constructor to initialize state and callbacks vector
3. Update `get()` to use `self.api.get_state()`
4. Update `update()` to:
   - Persist to storage via `self.api.update_state()`
   - Compare old and new state values
   - Update `current_state`
   - Call `fire_state_changed()` if values differ
5. Implement `subscribe()` with proper closure handling:
   - Create `Closure::wrap` for the callback
   - Register with TypeScript via `on_state_changed()`
   - **Store closure in `self.callbacks` vector** (critical for keeping it alive)
6. Implement `reset_subscriptions()` to:
   - Clear event listeners in TypeScript
   - Clear the `callbacks` vector to drop all closures
7. Add helper methods to get/set values in the `State` struct for comparison

### Step 4: Update vs_code_cargo_tools.rs

Update the initialization to properly construct `VSCodeStateManager`:
```rust
let state_manager = VSCodeStateManager::new();
```

### Step 5: Handle Async Callback Challenges

Since `AsyncFn` is not standard in stable Rust, consider:
- Using `Fn(&State) -> Pin<Box<dyn Future<Output = ()> + 'static>>`
- Or using a simpler synchronous callback signature
- Or using `wasm_bindgen_futures::spawn_local` to handle async execution

## Validation & Testing

### Build Commands

```bash
# Build WASM module
cargo build --package cargo_tools_vscode --target wasm32-unknown-unknown

# Or use cargo-make
cargo make compile

# Run tests
cargo test --package cargo_tools

# Lint
cargo clippy --package cargo_tools_vscode --target wasm32-unknown-unknown
```

### Integration Testing

1. Test state persistence across extension reloads
2. Test that subscribers are notified only when state actually changes
3. Test that subscribers are NOT notified when state is set to the same value
4. Test subscription cleanup on reset
5. Test multiple subscriptions receiving the same notification
6. Verify proper comparison of state values (consider implementing `PartialEq` for state types)

### Success Criteria

- [ ] `VSCodeStateManager` has no `todo!()` implementations
- [ ] State can be read and written via the `StateManager` trait
- [ ] `VSCodeStateManager` maintains a copy of the current `State`
- [ ] State updates trigger change detection by comparing old vs new values
- [ ] Subscriptions are properly registered with the TypeScript event emitter
- [ ] Subscribers are notified only when state values actually change
- [ ] `reset_subscriptions()` clears all event listeners
- [ ] Code compiles for `wasm32-unknown-unknown` target
- [ ] All tests pass (`cargo test --package cargo_tools`)
- [ ] Code passes clippy without warnings
- [ ] TypeScript `StateManagerApi` class uses `EventEmitter`
- [ ] TypeScript integration works correctly

## Architectural Constraints

Following the project's architectural guidelines:

1. **Trait Abstraction**: All VS Code API access must go through the `vs_code_api` module
2. **WASM Compatibility**: Use `#[wasm_async_trait]` for async trait implementations
3. **Error Handling**: Use `thiserror` for error types with explicit `#[source]` attributes
4. **Testing**: Core logic should be testable with mock implementations
5. **Documentation**: Comprehensive rustdoc for all public APIs

## References

- VS Code Extension API: https://code.visualstudio.com/api/references/vscode-api
- wasm-bindgen Guide: https://rustwasm.github.io/wasm-bindgen/
- async-trait: https://docs.rs/async-trait/latest/async_trait/
- Closures in wasm-bindgen: https://rustwasm.github.io/wasm-bindgen/reference/closure.html

## Notes

- The `AsyncFn` trait in the signature may need to be adapted to work with wasm-bindgen closures
- Consider using `js_sys::Function` directly if `AsyncFn` causes issues
- **Critical:** Closures MUST be stored in the struct, not forgotten - see [wasm-bindgen closure docs](https://rustwasm.github.io/wasm-bindgen/reference/closure.html)
- Do NOT use `Closure::forget()` - it causes memory leaks
- Store closures in a `Vec<Closure<...>>` and clear the vector in `reset_subscriptions()`
- Event listeners are cleared via `EventEmitter.removeAllListeners()` in TypeScript
- The Rust side should maintain a copy of `State` for change detection
- Consider implementing `PartialEq` and `Clone` for all state value types to enable comparison
- The TypeScript side needs access to `ExtensionContext` - ensure it's properly initialized
- Change detection happens in Rust, not TypeScript - this keeps logic in one place
- The TypeScript `EventEmitter` pattern is simpler than managing subscription IDs manually
- Dropping closures (via `clear()`) properly cleans up their resources