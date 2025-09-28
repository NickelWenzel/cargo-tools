// A simple proc-macro crate for testing
use proc_macro::TokenStream;

#[proc_macro]
pub fn hello_macro(_input: TokenStream) -> TokenStream {
    "fn hello() { println!(\"Hello, world!\"); }"
        .parse()
        .unwrap()
}
