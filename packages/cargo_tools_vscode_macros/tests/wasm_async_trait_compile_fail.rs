// Test file for compile-time failures of wasm_async_trait macro
// This file is NOT meant to compile - it tests error messages

use cargo_tools_vscode_macros::wasm_async_trait;

// Test: Applying to a function should fail
#[wasm_async_trait]
fn invalid_function() {}

// Test: Applying to a struct should fail
#[wasm_async_trait]
struct InvalidStruct;

// Test: Applying to an enum should fail
#[wasm_async_trait]
enum InvalidEnum {
    Variant,
}

// Test: Applying to a const should fail
#[wasm_async_trait]
const INVALID_CONST: i32 = 42;
