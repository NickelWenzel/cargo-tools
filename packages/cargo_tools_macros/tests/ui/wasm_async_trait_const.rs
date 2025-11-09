use cargo_tools_macros::wasm_async_trait;

// Error: wasm_async_trait can only be applied to trait definitions or trait implementations
#[wasm_async_trait]
const INVALID_CONST: i32 = 42;

fn main() {}
