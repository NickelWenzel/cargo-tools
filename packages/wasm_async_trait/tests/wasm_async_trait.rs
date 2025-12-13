use wasm_async_trait::wasm_async_trait;
use wasm_bindgen_test::wasm_bindgen_test;

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

#[wasm_bindgen_test(unsupported = tokio::test)]
async fn test_trait() {
    assert!(TestImpl.test_method().await == Ok("test".to_string()));
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
    T: Clone + Send + 'static,
{
    async fn generic_method(&self, value: T) -> Result<T, String> {
        Ok(value.clone())
    }
}

#[wasm_bindgen_test(unsupported = tokio::test)]
async fn test_generic_trait() {
    assert!(GenericImpl.generic_method(1).await == Ok(1));
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

#[wasm_bindgen_test(unsupported = tokio::test)]
async fn test_associated_type_trait() {
    assert!(AssociatedTypeImpl.process().await == Ok("processed".to_string()));
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

#[wasm_bindgen_test(unsupported = tokio::test)]
async fn test_multi_method_trait() {
    assert!(MultiMethodImpl.method_one().await == Ok(42));
    assert!(MultiMethodImpl.method_two(42).await == Ok("Value: 42".to_string()));
    assert!(MultiMethodImpl.sync_method() == 100);
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

#[wasm_bindgen_test(unsupported = tokio::test)]
async fn test_where_clause_trait() {
    assert!(WhereClauseImpl.process(10).await == Ok(10));
}
