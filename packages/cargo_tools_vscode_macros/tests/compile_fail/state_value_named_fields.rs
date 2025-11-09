use cargo_tools_vscode_macros::StateValue;

// Error: StateValue can only be derived for tuple structs, not structs with named fields
#[derive(StateValue)]
struct NamedFields {
    value: String,
}

fn main() {}
