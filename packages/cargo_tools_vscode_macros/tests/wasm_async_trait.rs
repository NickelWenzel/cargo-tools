use cargo_tools_vscode_macros::wasm_async_trait;

#[wasm_async_trait]
trait TestTrait {
    async fn test_method(&self) -> Result<String, String>;
}

struct TestImpl;

#[wasm_async_trait]
impl TestTrait for TestImpl {
    async fn test_method(&self) -> Result<String, String> {
        Ok("test".to_string())
    }
}

#[test]
fn test_trait_compiles() {
    // This test verifies that the macro generates valid code
    // The actual async_trait expansion is tested by the async-trait crate
}

// Test that it works with generic parameters
#[wasm_async_trait]
trait GenericTrait<T> {
    async fn generic_method(&self, value: T) -> Result<T, String>;
}

struct GenericImpl;

#[wasm_async_trait]
impl<T> GenericTrait<T> for GenericImpl
where
    T: Clone + 'static,
{
    async fn generic_method(&self, value: T) -> Result<T, String> {
        Ok(value.clone())
    }
}

#[test]
fn test_generic_trait_compiles() {
    // This test verifies that the macro works with generic parameters
}

// Test with associated types
#[wasm_async_trait]
trait AssociatedTypeTrait {
    type Output;
    type Error;

    async fn process(&self) -> Result<Self::Output, Self::Error>;
}

struct AssociatedTypeImpl;

#[wasm_async_trait]
impl AssociatedTypeTrait for AssociatedTypeImpl {
    type Output = String;
    type Error = String;

    async fn process(&self) -> Result<Self::Output, Self::Error> {
        Ok("processed".to_string())
    }
}

#[test]
fn test_associated_type_trait_compiles() {
    // This test verifies that the macro works with associated types
}

// Test with multiple methods
#[wasm_async_trait]
trait MultiMethodTrait {
    async fn method_one(&self) -> Result<i32, String>;
    async fn method_two(&self, value: i32) -> Result<String, String>;
    fn sync_method(&self) -> i32;
}

struct MultiMethodImpl;

#[wasm_async_trait]
impl MultiMethodTrait for MultiMethodImpl {
    async fn method_one(&self) -> Result<i32, String> {
        Ok(42)
    }

    async fn method_two(&self, value: i32) -> Result<String, String> {
        Ok(format!("Value: {}", value))
    }

    fn sync_method(&self) -> i32 {
        100
    }
}

#[test]
fn test_multi_method_trait_compiles() {
    // This test verifies that the macro works with multiple methods
}

// Test with where clauses
#[wasm_async_trait]
trait WhereClauseTrait<T>
where
    T: Clone + Send + 'static,
{
    async fn process(&self, value: T) -> Result<T, String>;
}

struct WhereClauseImpl;

#[wasm_async_trait]
impl<T> WhereClauseTrait<T> for WhereClauseImpl
where
    T: Clone + Send + 'static,
{
    async fn process(&self, value: T) -> Result<T, String> {
        Ok(value.clone())
    }
}

#[test]
fn test_where_clause_trait_compiles() {
    // This test verifies that the macro works with where clauses
}
