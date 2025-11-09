---
description: 'Rust programming language coding conventions and best practices'
applyTo: '**/*.rs'
---

# Rust Coding Conventions and Best Practices

Follow idiomatic Rust practices and community standards when writing Rust code. 

These instructions are based on [The Rust Book](https://doc.rust-lang.org/book/), [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), [RFC 430 naming conventions](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md), and the broader Rust community at [users.rust-lang.org](https://users.rust-lang.org).

## General Instructions

- Always prioritize readability, safety, and maintainability.
- Use strong typing and leverage Rust's ownership system for memory safety.
- Break down complex functions into smaller, more manageable functions.
- For algorithm-related code, include explanations of the approach used.
- Write code with good maintainability practices, including comments on why certain design decisions were made.
- Handle errors gracefully using `Result<T, E>` and provide meaningful error messages.
- For external dependencies, mention their usage and purpose in documentation.
- Use consistent naming conventions following [RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md).
- Write idiomatic, safe, and efficient Rust code that follows the borrow checker's rules.
- Ensure code compiles without warnings.

## Patterns to Follow

- Use modules (`mod`) and public interfaces (`pub`) to encapsulate logic.
- **Always use inline module files** (e.g., `my_module.rs`) instead of `mod.rs` files—declare submodules directly within the parent module file.
- Handle errors properly using `?`, `match`, or `if let`.
- Use `serde` for serialization and `thiserror` for custom errors.
- Implement traits to abstract services or external dependencies.
- Structure async code using `async/await`.
- Prefer enums over flags and states for type safety.
- Use builders for complex object creation.
- Split binary and library code (`main.rs` vs `lib.rs`) for testability and reuse.
- Use `rayon` for data parallelism and CPU-bound tasks.
- Use iterators instead of index-based loops as they're often faster and safer.
- Use `&str` instead of `String` for function parameters when you don't need ownership.
- Prefer borrowing and zero-copy operations to avoid unnecessary allocations.

### Ownership, Borrowing, and Lifetimes

- Prefer borrowing (`&T`) over cloning unless ownership transfer is necessary.
- Use `&mut T` when you need to modify borrowed data.
- Explicitly annotate lifetimes when the compiler cannot infer them.
- Use `Rc<T>` for single-threaded reference counting and `Arc<T>` for thread-safe reference counting.
- Use `RefCell<T>` for interior mutability in single-threaded contexts and `Mutex<T>` or `RwLock<T>` for multi-threaded contexts.

## Patterns to Avoid

- Don't use `unwrap()` or `expect()` unless absolutely necessary—prefer proper error handling.
- Avoid panics in library code—return `Result` instead.
- Don't rely on global mutable state—use dependency injection or thread-safe containers.
- Avoid deeply nested logic—refactor with functions or combinators.
- Don't ignore warnings—treat them as errors during CI.
- Avoid `unsafe` unless required and fully documented.
- Don't overuse `clone()`, use borrowing instead of cloning unless ownership transfer is needed.
- Avoid premature `collect()`, keep iterators lazy until you actually need the collection.
- Avoid unnecessary allocations—prefer borrowing and zero-copy operations.
- **Never create `mod.rs` files**—use inline module files (e.g., `my_module.rs`) and declare submodules within parent files.

## Code Style and Formatting

- Follow the Rust Style Guide and use `rustfmt` for automatic formatting.
- Keep lines under 100 characters when possible.
- Place function and struct documentation immediately before the item using `///`.
- Use `cargo clippy` to catch common mistakes and enforce best practices.

## Build and Compilation

### WASM-specific Crate (cargo_tools_vscode)
- **The `cargo_tools_vscode` crate is WASM-only** and should only be compiled for the `wasm32-unknown-unknown` target:
  ```sh
  cargo build -p cargo_tools_vscode --target wasm32-unknown-unknown
  ```
- Use `wasm-pack` for building and packaging this crate for JavaScript/TypeScript integration.

### Platform-independent Crates (cargo_tools, cargo_tools_macros, etc.)
- **All other crates must compile for both WASM and native targets**:
  ```sh
  # Build for native target
  cargo build -p <crate_name>
  
  # Build for WASM target
  cargo build -p <crate_name> --target wasm32-unknown-unknown
  ```
- **Run tests on the native target only**:
  ```sh
  cargo test -p <crate_name>
  ```
- Ensure code compiles without warnings on both target platforms.
- Use conditional compilation (`#[cfg(target_arch = "wasm32")]`) when platform-specific code is necessary.

## Error Handling

- Use `Result<T, E>` for recoverable errors and `panic!` only for unrecoverable errors.
- Prefer `?` operator over `unwrap()` or `expect()` for error propagation.
- Create custom error types using `thiserror`.
- Use `Option<T>` for values that may or may not exist.
- Provide meaningful error messages and context.
- Error types should be meaningful and well-behaved (implement standard traits).
- Validate function arguments and return appropriate errors for invalid input.

### Error Type Design with thiserror

When using `thiserror` for custom error types:
- **Always derive `thiserror::Error` on custom error types**
- **Always explicitly declare `#[source]` attributes** for error variants that wrap other errors
- **Do not implement `From` traits for error conversions**—rely only on `#[source]` for error chaining
- Do not rely on automatic source detection—make error mappings explicit
- This ensures clear error chains and prevents unexpected behavior when error types change

Example:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO operation failed")]
    Io(#[source] std::io::Error),  // Explicit #[source] required
    
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        #[source]
        source: serde_json::Error,  // Explicit #[source] required
    },
    
    #[error("Custom error: {0}")]
    Custom(String),  // No wrapped error, no #[source] needed
}
```

## API Design Guidelines

### Common Traits Implementation
Eagerly implement common traits where appropriate:
- `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Display`, `Default`
- Use standard conversion traits: `From`, `AsRef`, `AsMut`
- Collections should implement `FromIterator` and `Extend`
- Note: `Send` and `Sync` are auto-implemented by the compiler when safe; avoid manual implementation unless using `unsafe` code

### Type Safety and Predictability
- Use newtypes to provide static distinctions
- Arguments should convey meaning through types; prefer specific types over generic `bool` parameters
- Use `Option<T>` appropriately for truly optional values
- Functions with a clear receiver should be methods
- Only smart pointers should implement `Deref` and `DerefMut`

### Future Proofing
- Use sealed traits to protect against downstream implementations
- Structs should have private fields
- Functions should validate their arguments
- All public types must implement `Debug`

## Testing and Documentation

- Write comprehensive unit tests using `#[cfg(test)]` modules and `#[test]` annotations.
- Use test modules alongside the code they test (`mod tests { ... }`).
- Write integration tests in `tests/` directory with descriptive filenames.
- Write clear and concise comments for each function, struct, enum, and complex logic.
- Ensure functions have descriptive names and include comprehensive documentation.
- Document all public APIs with rustdoc (`///` comments) following the [API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `#[doc(hidden)]` to hide implementation details from public documentation.
- Document error conditions, panic scenarios, and safety considerations.
- Examples should use `?` operator, not `unwrap()` or deprecated `try!` macro.

### Compile-Fail Tests with trybuild

For procedural macros and type-level APIs that should reject invalid code at compile time, use the `trybuild` crate to write compile-fail tests:

- Add `trybuild` as a dev-dependency in `Cargo.toml`
- Create test files in `tests/compile_fail/` directory with `.rs` extension
- Create corresponding `.stderr` files with expected compiler error messages
- Use `#[test]` functions that call `trybuild::TestCases::new().compile_fail()`

**Example setup in `Cargo.toml`:**
```toml
[dev-dependencies]
trybuild = "1.0"
```

**Example test runner in `tests/compile_fail.rs`:**
```rust
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

**Example failing code in `tests/compile_fail/invalid_usage.rs`:**
```rust
use my_crate::MyMacro;

#[MyMacro]
fn invalid_function() {}  // This should fail to compile
```

**Expected output in `tests/compile_fail/invalid_usage.stderr`:**
```
error: MyMacro can only be applied to structs
 --> tests/compile_fail/invalid_usage.rs:3:1
  |
3 | #[MyMacro]
  | ^^^^^^^^^^
```

**Benefits of trybuild:**
- Ensures error messages are helpful and accurate
- Prevents regressions in compile-time error detection
- Documents expected failure modes
- Validates that invalid code is properly rejected

**When to use compile-fail tests:**
- Procedural macros with usage restrictions
- Derive macros with requirements on struct/enum shape
- Type-level APIs that enforce constraints at compile time
- Generic functions with trait bound requirements
- APIs designed to make incorrect usage impossible

## Project Organization

- Use semantic versioning in `Cargo.toml`.
- Include comprehensive metadata: `description`, `license`, `repository`, `keywords`, `categories`.
- Use feature flags for optional functionality.
- Organize code into modules using named files.
- Keep `main.rs` or `lib.rs` minimal - move logic to modules.

## Quality Checklist

Before publishing or reviewing Rust code, ensure:

### Core Requirements
- [ ] **Naming**: Follows RFC 430 naming conventions
- [ ] **Traits**: Implements `Debug`, `Clone`, `PartialEq` where appropriate
- [ ] **Error Handling**: Uses `Result<T, E>` and provides meaningful error types
- [ ] **Error Sources**: Explicit `#[source]` attributes for all wrapped errors in `thiserror` types
- [ ] **Documentation**: All public items have rustdoc comments with examples
- [ ] **Testing**: Comprehensive test coverage including edge cases

### Safety and Quality
- [ ] **Safety**: No unnecessary `unsafe` code, proper error handling
- [ ] **Performance**: Efficient use of iterators, minimal allocations
- [ ] **API Design**: Functions are predictable, flexible, and type-safe
- [ ] **Future Proofing**: Private fields in structs, sealed traits where appropriate
- [ ] **Tooling**: Code passes `cargo fmt`, `cargo clippy`, and `cargo test`