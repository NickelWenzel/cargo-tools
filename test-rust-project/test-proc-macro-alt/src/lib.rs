// Another proc-macro crate for testing (using crate-type syntax)
use proc_macro::TokenStream;

#[proc_macro_derive(HelloWorld)]
pub fn hello_world_derive(_input: TokenStream) -> TokenStream {
    "impl HelloWorld for Test { fn hello_world(&self) { println!(\"Hello, World!\"); } }"
        .parse()
        .unwrap()
}
