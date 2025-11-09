use cargo_tools_vscode_macros::StateValue;

// Error: StateValue can only be derived for tuple structs, not enums
#[derive(StateValue)]
enum MyEnum {
    Variant1,
    Variant2,
}

fn main() {}
