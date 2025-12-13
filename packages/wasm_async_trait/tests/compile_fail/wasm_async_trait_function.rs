use wasm_async_trait::wasm_async_trait;

// Error: wasm_async_trait can only be applied to trait definitions or trait implementations
#[wasm_async_trait]
fn invalid_function() {}

fn main() {}
