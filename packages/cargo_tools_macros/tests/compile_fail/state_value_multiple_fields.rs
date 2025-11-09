use cargo_tools_macros::StateValue;

// Error: StateValue can only be derived for tuple structs with exactly one field
#[derive(StateValue)]
struct MultipleFields(String, i32);

fn main() {}
