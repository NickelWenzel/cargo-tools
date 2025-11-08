# cargo_tools_macros

Procedural macros for the cargo-tools VS Code extension.

## StateValue Derive Macro

The `StateValue` derive macro automatically implements the `StateValue` trait for newtype wrapper structs used in the state management system.

### Usage

```rust
use cargo_tools_macros::StateValue;

#[derive(StateValue)]
pub struct SelectedPackage(String);

#[derive(StateValue)]
pub struct GroupByWorkspaceMember(bool);

#[derive(StateValue)]
pub struct SelectedFeatures(Vec<String>);
```

### Generated Implementation

The macro generates:

1. **KEY constant**: Converts the struct name from PascalCase to camelCase
   - `SelectedPackage` → `"selectedPackage"`
   - `GroupByWorkspaceMember` → `"groupByWorkspaceMember"`

2. **Value associated type**: Inferred from the wrapped type
   - `SelectedPackage(String)` → `type Value = String`
   - `SelectedFeatures(Vec<String>)` → `type Value = Vec<String>`

3. **into_value method**: Returns the wrapped value via `self.0`

### Requirements

- Must be applied to tuple structs with exactly one field
- The wrapped type can be any type (primitives, collections, custom types)

### Example Expansion

```rust
#[derive(StateValue)]
pub struct SelectedPackage(String);
```

Expands to:

```rust
impl StateValue for SelectedPackage {
    const KEY: &'static str = "selectedPackage";
    type Value = String;
    
    fn into_value(self) -> Self::Value {
        self.0
    }
}
```

### Error Handling

The macro provides clear error messages for invalid usage:

- Multiple fields: "StateValue can only be derived for tuple structs with exactly one field"
- Named fields: "StateValue can only be derived for tuple structs, not structs with named fields"
- Unit structs: "StateValue can only be derived for tuple structs with exactly one field, not unit structs"
- Enums/Unions: "StateValue can only be derived for tuple structs, not enums/unions"

## Development

Run tests:
```bash
cargo test -p cargo_tools_macros
```

Run tests with the full workspace:
```bash
cargo make test
```
