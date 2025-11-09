# cargo_tools_macros

Procedural macros for the cargo-tools VS Code extension.

## Macros

### wasm_async_trait

The `wasm_async_trait` attribute macro automatically applies the correct `async_trait` variant based on the compilation target, allowing the same async trait code to work seamlessly across both WASM and native targets.

#### The Problem

The standard `async_trait` macro adds a `Send` bound to generated futures by default. However:
- WASM targets are single-threaded and many WASM types (from `wasm_bindgen`) cannot implement `Send`
- This requires manually using `#[async_trait(?Send)]` for WASM code
- Conditional compilation attributes would need to be scattered throughout the codebase

#### The Solution

`wasm_async_trait` automatically selects the appropriate variant:
- **WASM targets (wasm32)**: Uses `#[async_trait(?Send)]` (no Send requirement)
- **Native targets**: Uses `#[async_trait]` (with Send bound)

#### Usage

Apply to both trait definitions and implementations:

```rust
use cargo_tools_macros::wasm_async_trait;

#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;
    
    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue + 'static>(
        &self,
        value: T,
    ) -> Result<(), Self::UpdateError>;
}

#[wasm_async_trait]
impl StateManager for VSCodeStateManager {
    type UpdateError = StateManagerError;
    
    fn get<T: StateValue>(&self) -> Option<T> {
        // ...
    }
    
    async fn update<T: StateValue + 'static>(
        &self,
        value: T,
    ) -> Result<(), Self::UpdateError> {
        // ...
    }
}
```

#### Expansion

**On WASM targets (wasm32):**
```rust
#[async_trait(?Send)]
pub trait StateManager {
    // ...
}

#[async_trait(?Send)]
impl StateManager for VSCodeStateManager {
    // ...
}
```

**On native targets:**
```rust
#[async_trait]
pub trait StateManager {
    // ...
}

#[async_trait]
impl StateManager for VSCodeStateManager {
    // ...
}
```

#### Features

- **Transparent**: Works as a drop-in replacement for `#[async_trait]`
- **Cross-platform**: Single codebase works on both WASM and native targets
- **Type-safe**: Preserves stronger guarantees (Send bounds) on native platforms
- **Generic support**: Works with generic parameters, where clauses, and associated types
- **Clear errors**: Provides helpful error messages for invalid usage

#### Error Handling

The macro can only be applied to trait definitions or trait implementations:

```rust
// ❌ Error: Invalid usage
#[wasm_async_trait]
fn invalid_function() {}

// ❌ Error: Invalid usage
#[wasm_async_trait]
struct InvalidStruct;
```

Error message:
```
wasm_async_trait can only be applied to trait definitions or trait implementations
```

## StateValue Derive Macro

The `StateValue` derive macro automatically implements the `StateValue` trait for newtype wrapper structs used in the state management system.

### Usage

```rust
use cargo_tools_macros::StateValue;

#[derive(StateValue)]
pub struct SelectedPackage(String);

#[derive(StateValue)]
pub struct GroupByWorkspaceMember(bool);

#[derive(StateValue)]
pub struct SelectedFeatures(Vec<String>);
```

### Generated Implementation

The macro generates:

1. **KEY constant**: Converts the struct name from PascalCase to camelCase
   - `SelectedPackage` → `"selectedPackage"`
   - `GroupByWorkspaceMember` → `"groupByWorkspaceMember"`

2. **Value associated type**: Inferred from the wrapped type
   - `SelectedPackage(String)` → `type Value = String`
   - `SelectedFeatures(Vec<String>)` → `type Value = Vec<String>`

3. **into_value method**: Returns the wrapped value via `self.0`

### Requirements

- Must be applied to tuple structs with exactly one field
- The wrapped type can be any type (primitives, collections, custom types)

### Example Expansion

```rust
#[derive(StateValue)]
pub struct SelectedPackage(String);
```

Expands to:

```rust
impl StateValue for SelectedPackage {
    const KEY: &'static str = "selectedPackage";
    type Value = String;
    
    fn into_value(self) -> Self::Value {
        self.0
    }
}
```

### Error Handling

The macro provides clear error messages for invalid usage:

- Multiple fields: "StateValue can only be derived for tuple structs with exactly one field"
- Named fields: "StateValue can only be derived for tuple structs, not structs with named fields"
- Unit structs: "StateValue can only be derived for tuple structs with exactly one field, not unit structs"
- Enums/Unions: "StateValue can only be derived for tuple structs, not enums/unions"

## Development

Run tests:
```bash
cargo test -p cargo_tools_macros
```

Run tests with the full workspace:
```bash
cargo make test
```
