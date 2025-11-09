use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Item};

/// Derive macro for the StateValue trait.
///
/// This macro automatically implements the `StateValue` trait for newtype wrapper structs.
/// It generates:
/// - A `KEY` constant by converting the struct name from PascalCase to camelCase
/// - A `Value` associated type inferred from the wrapped type
/// - An `into_value` method that returns the wrapped value
///
/// # Requirements
///
/// The macro can only be applied to tuple structs with exactly one field.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(StateValue)]
/// pub struct SelectedPackage(String);
///
/// // Expands to:
/// // impl StateValue for SelectedPackage {
/// //     const KEY: &'static str = "selectedPackage";
/// //     type Value = String;
/// //     
/// //     fn into_value(self) -> Self::Value {
/// //         self.0
/// //     }
/// // }
/// ```
#[proc_macro_derive(StateValue)]
pub fn derive_state_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Extract the wrapped type from the tuple struct
    let wrapped_type = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    struct_name,
                    "StateValue can only be derived for tuple structs with exactly one field",
                )
                .to_compile_error()
                .into();
            }
            Fields::Named(_) => {
                return syn::Error::new_spanned(
                    struct_name,
                    "StateValue can only be derived for tuple structs, not structs with named fields",
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(
                    struct_name,
                    "StateValue can only be derived for tuple structs with exactly one field, not unit structs",
                )
                .to_compile_error()
                .into();
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "StateValue can only be derived for tuple structs, not enums",
            )
            .to_compile_error()
            .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "StateValue can only be derived for tuple structs, not unions",
            )
            .to_compile_error()
            .into();
        }
    };

    // Convert struct name from PascalCase to camelCase
    let key = to_camel_case(&struct_name.to_string());

    // Generate the trait implementation
    let expanded = quote! {
        impl StateValue for #struct_name {
            const KEY: &'static str = #key;
            type Value = #wrapped_type;

            fn into_value(self) -> Self::Value {
                self.0
            }
        }
    };

    TokenStream::from(expanded)
}

/// Convert a PascalCase string to camelCase.
///
/// Examples:
/// - `SelectedPackage` → `"selectedPackage"`
/// - `GroupByWorkspaceMember` → `"groupByWorkspaceMember"`
/// - `IsTargetTypeFilterActive` → `"isTargetTypeFilterActive"`
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

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
/// use cargo_tools_vscode_macros::wasm_async_trait;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("SelectedPackage"), "selectedPackage");
        assert_eq!(
            to_camel_case("GroupByWorkspaceMember"),
            "groupByWorkspaceMember"
        );
        assert_eq!(
            to_camel_case("IsTargetTypeFilterActive"),
            "isTargetTypeFilterActive"
        );
        assert_eq!(to_camel_case("ShowFeatures"), "showFeatures");
        assert_eq!(to_camel_case("A"), "a");
        assert_eq!(to_camel_case(""), "");
    }
}
