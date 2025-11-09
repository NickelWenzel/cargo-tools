---
mode: agent
model: Claude Sonnet 4.5 (copilot)
tools: ['search', 'edit', 'fetch']
description: 'Implement WASM-aware async_trait Macro'
---

# Task: Implement WASM-aware async_trait Macro

## Objective

Create a procedural attribute macro `#[wasm_async_trait]` that automatically applies the correct `async_trait` variant based on the compilation target:
- For WASM targets: uses `#[async_trait(?Send)]` (single-threaded, no Send requirement)
- For all other targets: uses `#[async_trait]` (default with Send bound)

## Context

The standard `async_trait` macro from the `async-trait` crate adds a `Send` bound to generated futures by default. However, WASM targets are single-threaded and many WASM types (like those from `wasm_bindgen`) cannot implement `Send`. This requires manually using `#[async_trait(?Send)]` for WASM code.

This macro automates the selection, allowing the same trait definition to work across both native and WASM targets without conditional compilation attributes scattered throughout the codebase.

## Requirements

### Macro Behavior

1. **Target Detection**: Use `cfg(target_arch = "wasm32")` to detect WASM compilation
2. **Automatic Delegation**: 
   - On WASM targets: expand to `#[async_trait(?Send)]`
   - On non-WASM targets: expand to `#[async_trait]`
3. **Transparent**: Should work as a drop-in replacement for `#[async_trait]`
4. **Both Trait and Impl**: Must work on both trait definitions and implementations

### Usage Example

```rust
use cargo_tools_macros::wasm_async_trait;

// Works on trait definitions
#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;
    
    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue + 'static>(
        &self,
        value: T,
    ) -> Result<(), Self::UpdateError>;
}

// Works on trait implementations
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

Should expand to:

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

**On non-WASM targets:**
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

## Implementation Steps

### 1. Update Macro Crate

Add `async-trait` dependency to `packages/cargo_tools_macros/Cargo.toml`:

```toml
[dependencies]
syn = { workspace = true, features = ["full"] }
quote = { workspace = true }
proc-macro2 = { workspace = true }
async-trait = { workspace = true }
```

Update workspace `Cargo.toml`:

```toml
[workspace.dependencies]
async-trait = "0.1"
```

### 2. Implement the Macro

In `packages/cargo_tools_macros/src/lib.rs`:

```rust
/// Automatically applies the correct async_trait variant based on compilation target.
///
/// On WASM targets (wasm32), this expands to `#[async_trait(?Send)]` since WASM is
/// single-threaded and many WASM types cannot implement Send.
///
/// On all other targets, this expands to `#[async_trait]` with the default Send bound.
///
/// This allows writing async traits that work across both native and WASM targets
/// without manual conditional compilation.
///
/// # Examples
///
/// ```rust
/// use cargo_tools_macros::wasm_async_trait;
///
/// #[wasm_async_trait]
/// pub trait MyTrait {
///     async fn my_method(&self) -> Result<(), String>;
/// }
///
/// struct MyImpl;
///
/// #[wasm_async_trait]
/// impl MyTrait for MyImpl {
///     async fn my_method(&self) -> Result<(), String> {
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_async_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse input to ensure it's valid Rust
    let input = parse_macro_input!(input as DeriveInput);
    
    // Generate the appropriate async_trait attribute based on target
    let expanded = quote! {
        #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
        #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
        #input
    };
    
    TokenStream::from(expanded)
}
```

**Note:** The actual implementation needs to handle both trait definitions and trait implementations. The above is a simplified example showing the concept.

### 3. Proper Implementation Details

The macro should:
- Accept both `trait` and `impl` items as input
- Forward any arguments passed to the macro to the underlying `async_trait` (though typically none are needed)
- Preserve all attributes, generics, where clauses, and other syntax elements
- Use `syn::Item` to parse either `ItemTrait` or `ItemImpl`
- Generate appropriate `cfg_attr` conditional compilation attributes

Example proper implementation structure:

```rust
#[proc_macro_attribute]
pub fn wasm_async_trait(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    
    let expanded = quote! {
        #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
        #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
        #input
    };
    
    TokenStream::from(expanded)
}
```

### 4. Error Handling

Provide clear compiler errors for invalid usage:
- Must be applied to `trait` or `impl` blocks only
- Cannot be applied to functions, structs, or other items

Example error message:
```
error: wasm_async_trait can only be applied to trait definitions or trait implementations
  --> src/state_manager.rs:5:1
   |
5  | #[wasm_async_trait]
   | ^^^^^^^^^^^^^^^^^^^
```

### 5. Add Tests

Create `packages/cargo_tools_macros/tests/wasm_async_trait.rs`:

```rust
use cargo_tools_macros::wasm_async_trait;

#[wasm_async_trait]
trait TestTrait {
    async fn test_method(&self) -> Result<String, String>;
}

struct TestImpl;

#[wasm_async_trait]
impl TestTrait for TestImpl {
    async fn test_method(&self) -> Result<String, String> {
        Ok("test".to_string())
    }
}

#[test]
fn test_trait_compiles() {
    // This test verifies that the macro generates valid code
    // The actual async_trait expansion is tested by the async-trait crate
}

// Test that it works with generic parameters
#[wasm_async_trait]
trait GenericTrait<T> {
    async fn generic_method(&self, value: T) -> Result<T, String>;
}

struct GenericImpl;

#[wasm_async_trait]
impl<T> GenericTrait<T> for GenericImpl 
where
    T: Send + Sync + Clone,
{
    async fn generic_method(&self, value: T) -> Result<T, String> {
        Ok(value.clone())
    }
}
```

### 6. Integration

Update `packages/cargo_tools_vscode/Cargo.toml` to ensure `async-trait` is available:

```toml
[dependencies]
cargo_tools_macros = { path = "../cargo_tools_macros" }
async-trait = { workspace = true }
```

Update existing code in `packages/cargo_tools_vscode/src/state_manager.rs`:

```rust
use cargo_tools_macros::wasm_async_trait;

// Replace #[async_trait(?Send)] with #[wasm_async_trait]
#[wasm_async_trait]
pub trait StateManager {
    type UpdateError;

    fn get<T: StateValue>(&self) -> Option<T>;
    async fn update<T: StateValue + 'static>(
        &self,
        value: T,
    ) -> Result<(), Self::UpdateError>;
}
```

Update `packages/cargo_tools_vscode/src/vs_code_cargo_tools/state_manager.rs`:

```rust
use cargo_tools_macros::wasm_async_trait;

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

## Validation

Run these commands to validate the implementation:

```bash
# Test the macro crate specifically
cargo test -p cargo_tools_macros

# Build for WASM target to verify ?Send is applied
cargo build --target wasm32-unknown-unknown -p cargo_tools_vscode

# Build for native target to verify Send is applied
cargo build -p cargo_tools_vscode

# Run all tests
cargo make test

# Run linting
cargo make lint
```

## Success Criteria

- [ ] Macro compiles without warnings
- [ ] All tests in macro crate pass
- [ ] Code compiles successfully for both WASM and native targets
- [ ] `StateManager` trait and implementations use `#[wasm_async_trait]`
- [ ] Macro works with generic parameters and where clauses
- [ ] Macro works with associated types
- [ ] Clear error messages for invalid usage
- [ ] Code passes `cargo clippy`
- [ ] Documentation is clear and includes examples
- [ ] No conditional compilation needed at usage sites

## References

- [async-trait crate documentation](https://docs.rs/async-trait)
- [The Rust Book - Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Procedural Macros Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [syn crate documentation](https://docs.rs/syn)
- [quote crate documentation](https://docs.rs/quote)
- [Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)

## Architectural Alignment

This implementation follows the project's architectural guidelines:
- Reduces boilerplate and conditional compilation
- Maintains trait-based abstraction pattern
- Works seamlessly with existing WASM and native code
- Clear separation of concerns (macro crate vs. main crate)
- Comprehensive testing strategy
- Follows Rust coding conventions from `.github/instructions/rust_coding_guidelines.instructions.md`

## Additional Considerations

### Why Not Just Use `#[async_trait(?Send)]` Everywhere?

While `?Send` would work on both WASM and native targets, it removes thread-safety guarantees on native targets where they're valuable. This macro preserves the stronger guarantees (`Send` bounds) on native targets while relaxing them only where necessary (WASM).

### Alternative Approaches Considered

1. **Manual conditional compilation**: Requires `#[cfg(target_arch = "wasm32")]` at every usage site
2. **Always use `?Send`**: Loses thread-safety guarantees on native platforms
3. **Separate trait definitions**: Leads to code duplication and maintenance burden

The macro approach provides the best developer experience with proper semantics for each target.