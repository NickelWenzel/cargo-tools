# Coding Conventions

## Rust Guidelines (from rust_coding_guidelines.instructions.md)
- Follow idiomatic Rust, Rust Book, API Guidelines, RFC 430
- Use inline module files (e.g., `my_module.rs`), never `mod.rs`
- Handle errors with Result<T, E>, use thiserror for custom errors
- Always use explicit #[source] attributes in thiserror error types
- Do NOT implement From traits for error conversions
- Avoid unwrap()/expect(), prefer ? operator
- Use async/await for async code
- Implement common traits: Debug, Clone, PartialEq where appropriate
- Validate function arguments, provide meaningful errors
- Use rustfmt for formatting, clippy for linting
- Lines under 100 characters
- Comprehensive rustdoc comments on public APIs
- Write tests using #[test] and #[cfg(test)]

## Architecture Patterns
- **Trait Abstraction**: All functions/types requiring VS Code API must be abstracted by traits
- **Implementation Isolation**: Only concrete trait implementations depend on vs_code_api module
- **Test Independence**: Tests use mock trait implementations without vs_code_api dependencies
- **WASM Integration**: Core logic in Rust, VSCodeCargoTools wraps CargoTools for WASM bindings
- **Migration Strategy**: Move TypeScript logic to Rust incrementally

## Error Handling Pattern
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO operation failed")]
    Io(#[source] std::io::Error),  // Explicit #[source] required
}
```
