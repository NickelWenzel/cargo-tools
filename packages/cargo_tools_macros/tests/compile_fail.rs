/// Compile-fail tests for the StateValue and wasm_async_trait macros.
///
/// These tests verify that the macros properly reject invalid usage at compile time
/// and provide helpful error messages.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
