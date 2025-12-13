# wasm_async_trait

A procedural macro that automatically applies the correct `async_trait` variant based on compilation target, enabling async traits to work seamlessly across both WASM and native targets.

## The Problem

The standard `async_trait` macro adds a `Send` bound to generated futures by default. However:
- WASM targets are single-threaded and many WASM types (from `wasm_bindgen`) cannot implement `Send`
- This requires manually using `#[async_trait(?Send)]` for WASM code
- Conditional compilation attributes would need to be scattered throughout the codebase

## The Solution

`wasm_async_trait` automatically selects the appropriate variant:
- **WASM targets (wasm32)**: Uses `#[async_trait(?Send)]` (no Send requirement)
- **Native targets**: Uses `#[async_trait]` (with Send bound)

## Usage

Apply to both trait definitions and implementations:

```rust
use wasm_async_trait::wasm_async_trait;

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

## Expansion

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

## Features

- **Transparent**: Works as a drop-in replacement for `#[async_trait]`
- **Cross-platform**: Single codebase works on both WASM and native targets
- **Type-safe**: Preserves stronger guarantees (Send bounds) on native platforms
- **Generic support**: Works with generic parameters, where clauses, and associated types
- **Clear errors**: Provides helpful error messages for invalid usage

## Error Handling

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

## Development

Run tests:
```bash
cargo test -p wasm_async_trait
```

Run tests with the full workspace:
```bash
cargo make test
```
