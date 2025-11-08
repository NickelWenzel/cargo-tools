/// This file contains tests for invalid usage of the StateValue derive macro.
/// These are documented examples that should NOT compile.

// Uncomment the following to test compilation errors:

/*
use cargo_tools_macros::StateValue;

// Error: Multiple fields
#[derive(StateValue)]
struct MultipleFields(String, i32);

// Error: Named fields
#[derive(StateValue)]
struct NamedFields {
    value: String,
}

// Error: Unit struct
#[derive(StateValue)]
struct UnitStruct;

// Error: Empty tuple struct
#[derive(StateValue)]
struct EmptyTuple();

// Error: Enum
#[derive(StateValue)]
enum MyEnum {
    Variant1,
    Variant2,
}
*/

#[test]
fn test_compile_fail_documentation() {
    // This test exists to document expected compilation failures.
    // To test these, uncomment the code above and verify compilation errors.
}
