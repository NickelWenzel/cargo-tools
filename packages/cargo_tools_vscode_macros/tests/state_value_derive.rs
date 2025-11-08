use cargo_tools_vscode_macros::StateValue;

// Test structs with various types
#[derive(StateValue)]
struct TestState(String);

#[derive(StateValue)]
struct BooleanState(bool);

#[derive(StateValue)]
struct VectorState(Vec<String>);

#[derive(StateValue)]
struct NumericState(i32);

#[derive(StateValue)]
struct SelectedPackage(String);

#[derive(StateValue)]
struct GroupByWorkspaceMember(bool);

#[derive(StateValue)]
struct IsTargetTypeFilterActive(bool);

#[derive(StateValue)]
struct SelectedFeatures(Vec<String>);

// Define the trait for testing (would normally come from cargo_tools_vscode)
trait StateValue {
    const KEY: &'static str;
    type Value;

    fn into_value(self) -> Self::Value;
}

#[test]
fn test_key_generation() {
    assert_eq!(TestState::KEY, "testState");
    assert_eq!(BooleanState::KEY, "booleanState");
    assert_eq!(VectorState::KEY, "vectorState");
    assert_eq!(NumericState::KEY, "numericState");
}

#[test]
fn test_key_generation_complex_names() {
    assert_eq!(SelectedPackage::KEY, "selectedPackage");
    assert_eq!(GroupByWorkspaceMember::KEY, "groupByWorkspaceMember");
    assert_eq!(IsTargetTypeFilterActive::KEY, "isTargetTypeFilterActive");
    assert_eq!(SelectedFeatures::KEY, "selectedFeatures");
}

#[test]
fn test_value_type_string() {
    let state = TestState(String::from("test"));
    let value: String = state.into_value();
    assert_eq!(value, "test");
}

#[test]
fn test_value_type_bool() {
    let state = BooleanState(true);
    let value: bool = state.into_value();
    assert_eq!(value, true);
}

#[test]
fn test_value_type_vector() {
    let state = VectorState(vec![String::from("a"), String::from("b")]);
    let value: Vec<String> = state.into_value();
    assert_eq!(value, vec!["a", "b"]);
}

#[test]
fn test_value_type_numeric() {
    let state = NumericState(42);
    let value: i32 = state.into_value();
    assert_eq!(value, 42);
}

#[test]
fn test_into_value_selected_package() {
    let package = SelectedPackage(String::from("my-package"));
    let value: String = package.into_value();
    assert_eq!(value, "my-package");
}

#[test]
fn test_into_value_selected_features() {
    let features = SelectedFeatures(vec![String::from("feature1"), String::from("feature2")]);
    let value: Vec<String> = features.into_value();
    assert_eq!(value, vec!["feature1", "feature2"]);
}
