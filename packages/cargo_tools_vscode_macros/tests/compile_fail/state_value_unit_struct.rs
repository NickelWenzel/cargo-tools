use cargo_tools_vscode_macros::StateValue;

// Error: StateValue can only be derived for tuple structs with exactly one field, not unit structs
#[derive(StateValue)]
struct UnitStruct;

fn main() {}
