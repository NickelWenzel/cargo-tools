use cargo_tools_vscode_macros::StateValue;

// Error: StateValue can only be derived for tuple structs with exactly one field
#[derive(StateValue)]
struct EmptyTuple();

fn main() {}
