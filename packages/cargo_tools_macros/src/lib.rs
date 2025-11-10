use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

/// Automatically applies the correct `async_trait` variant based on compilation target.
///
/// On WASM targets (wasm32), this expands to `#[async_trait(?Send)]` since WASM is
/// single-threaded and many WASM types cannot implement `Send`.
///
/// On all other targets, this expands to `#[async_trait]` with the default `Send` bound.
///
/// This allows writing async traits that work across both native and WASM targets
/// without manual conditional compilation.
///
/// # Examples
///
/// ```rust,ignore
/// use cargo_tools_macros::wasm_async_trait;
///
/// #[wasm_async_trait]
/// pub trait MyTrait {
///     async fn my_method(&self) -> Result<(), String>;
/// }
///
/// struct MyImpl;
///
/// #[wasm_async_trait]
/// impl MyTrait for MyImpl {
///     async fn my_method(&self) -> Result<(), String> {
///         Ok(())
///     }
/// }
/// ```
///
/// # Error Handling
///
/// The macro can only be applied to trait definitions or trait implementations.
/// Applying it to other items (functions, structs, etc.) will result in a compile error.
#[proc_macro_attribute]
pub fn wasm_async_trait(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    // Validate that the input is either a trait or impl block
    match &input {
        Item::Trait(_) | Item::Impl(_) => {
            // Valid usage - generate the appropriate async_trait attributes
            let expanded = quote! {
                #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
                #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
                #input
            };
            TokenStream::from(expanded)
        }
        _ => {
            // Invalid usage - return a compiler error
            let error = syn::Error::new_spanned(
                &input,
                "wasm_async_trait can only be applied to trait definitions or trait implementations",
            );
            error.to_compile_error().into()
        }
    }
}
