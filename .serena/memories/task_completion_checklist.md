# Task Completion Checklist

When completing a task, ensure:

## Build & Compilation
- [ ] `cargo build -p <crate_name>` - Compiles without warnings
- [ ] `cargo build -p <crate_name> --target wasm32-unknown-unknown` - WASM compilation works (except cargo_tools_vscode)
- [ ] `cargo make compile` - Full extension build succeeds

## Testing
- [ ] `cargo test -p <crate_name>` - All tests pass
- [ ] `cargo make test` - Full test suite passes

## Linting & Formatting
- [ ] `cargo fmt` - Code formatted
- [ ] `cargo clippy` - No clippy warnings
- [ ] `cargo make lint` - ESLint + clippy pass

## Code Quality
- [ ] Error handling uses Result<T, E> with thiserror
- [ ] Explicit #[source] attributes for wrapped errors
- [ ] No unwrap()/expect() in library code
- [ ] Public APIs have rustdoc comments
- [ ] Tests cover new functionality
- [ ] Follows naming conventions (RFC 430)
- [ ] Trait abstractions for VS Code API dependencies

## Documentation
- [ ] Update relevant memory files if architecture changes
- [ ] Add rustdoc examples for public APIs
