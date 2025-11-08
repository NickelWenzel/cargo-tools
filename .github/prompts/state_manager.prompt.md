---
mode: agent
model: Claude Sonnet 4.5 (copilot)
tools: ['search', 'edit', 'fetch']
description: 'Implement StateValue Derive Macro'
---

# Task: Implement StateValue Derive Macro

## Objective

Create a procedural derive macro `#[derive(StateValue)]` that automatically implements the `StateValue` trait for newtype wrapper structs in the state management system.

## Context

The `StateValue` trait is currently defined in `packages/cargo_tools_vscode/src/state_manager.rs`:

```rust
pub trait StateValue {
    const KEY: &'static str;
    type Value;
    
    fn into_value(self) -> Self::Value;
}
```

Currently, implementations would need to be manual for each newtype wrapper, but we want to automate this with a derive macro.

## Requirements

### Macro Behavior

1. **Generate `KEY` constant**: Extract the struct name and convert it to camelCase
   - Convert from PascalCase to camelCase (e.g., `SelectedPackage` → `"selectedPackage"`)
   - No customization needed - always derive from struct name

2. **Infer `Value` type**: Extract the wrapped type from the newtype struct
   - For `SelectedPackage(String)`, the `Value` type should be `String`
   - Support any wrapped type (primitives, collections, custom types)

3. **Implement `into_value` method**: Return the wrapped value
   - For tuple structs: `self.0`
   - Should work with single-field tuple structs only

### Usage Example

```rust
#[derive(StateValue)]
pub struct SelectedPackage(String);

#[derive(StateValue)]
pub struct GroupByWorkspaceMember(bool);

#[derive(StateValue)]
pub struct SelectedFeatures(Vec<String>);
```

Should expand to:

```rust
impl StateValue for SelectedPackage {
    const KEY: &'static str = "selectedPackage";
    type Value = String;
    
    fn into_value(self) -> Self::Value {
        self.0
    }
}
```

## Implementation Steps

### 1. Create Macro Crate

Create a new workspace member at `packages/cargo_tools_macros/`:

```toml
# packages/cargo_tools_macros/Cargo.toml
[package]
name = "cargo_tools_macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { workspace = true, features = ["full"] }
quote = { workspace = true }
proc-macro2 = { workspace = true }
```

Update workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "packages/cargo_tools_vscode",
    "packages/cargo_tools_macros",
]

[workspace.dependencies]
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"
```

### 2. Implement the Macro

In `packages/cargo_tools_macros/src/lib.rs`:

- Parse struct definition using `syn`
- Validate it's a tuple struct with exactly one field
- Extract struct name and convert to camelCase
- Extract the wrapped type from the first field
- Generate trait implementation using `quote!`

### 3. Name Conversion Algorithm

Implement PascalCase to camelCase conversion:

```rust
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}
```

Examples:
- `SelectedPackage` → `"selectedPackage"`
- `GroupByWorkspaceMember` → `"groupByWorkspaceMember"`
- `IsTargetTypeFilterActive` → `"isTargetTypeFilterActive"`

### 4. Error Handling

Provide clear compiler errors for invalid usage:
- Structs with multiple fields
- Named structs (not tuple structs)
- Empty structs
- Unit structs

Example error message:
```
error: StateValue can only be derived for tuple structs with exactly one field
  --> src/state_manager.rs:5:1
   |
5  | #[derive(StateValue)]
   | ^^^^^^^^^^^^^^^^^^^^^
```

### 5. Add Tests

Create `packages/cargo_tools_macros/tests/state_value_derive.rs`:

```rust
use cargo_tools_macros::StateValue;

#[derive(StateValue)]
struct TestState(String);

#[derive(StateValue)]
struct BooleanState(bool);

#[test]
fn test_key_generation() {
    assert_eq!(TestState::KEY, "testState");
    assert_eq!(BooleanState::KEY, "booleanState");
}

#[test]
fn test_value_type() {
    let state = TestState(String::from("test"));
    let value: String = state.into_value();
    assert_eq!(value, "test");
}
```

### 6. Integration

Update `packages/cargo_tools_vscode/Cargo.toml`:

```toml
[dependencies]
cargo_tools_macros = { path = "../cargo_tools_macros" }
```

Update `packages/cargo_tools_vscode/src/state_manager.rs` to use the derive macro:

```rust
use cargo_tools_macros::StateValue;

#[derive(Debug, StateValue)]
pub struct SelectedPackage(String);

#[derive(Debug, StateValue)]
pub struct SelectedBuildTarget(String);

// ... apply to all state structs
```

## Validation

Run these commands to validate the implementation:

```bash
cargo make compile  # Ensure everything builds
cargo make test     # Run all tests including macro tests
cargo make lint     # Check code quality
```

## Success Criteria

- [ ] Macro compiles without warnings
- [ ] All tests in macro crate pass
- [ ] `state_manager.rs` successfully uses the derive macro
- [ ] All `StateValue` implementations generate correct KEY constants
- [ ] Type inference works for all wrapped types (String, bool, Vec<String>, etc.)
- [ ] Clear error messages for invalid usage
- [ ] Code passes `cargo clippy`
- [ ] No manual trait implementations remain in `state_manager.rs`

## References

- [The Rust Book - Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Procedural Macros Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [syn crate documentation](https://docs.rs/syn)
- [quote crate documentation](https://docs.rs/quote)
- [proc-macro2 crate documentation](https://docs.rs/proc-macro2)

## Architectural Alignment

This implementation follows the project's architectural guidelines:
- Trait-based abstraction for extensibility
- Clear separation of concerns (macro crate vs. main crate)
- Comprehensive testing strategy
- Follows Rust coding conventions from `.github/instructions/rust_coding_guidelines.instructions.md`