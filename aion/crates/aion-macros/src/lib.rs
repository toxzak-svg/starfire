//! Aion proc macros.

use proc_macro::TokenStream;

/// Derive macro for implementing the MindLogic trait.
/// Adds `new()`, `start()`, and `resume()` defaults.
#[proc_macro_derive(Mind)]
pub fn mind_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    quote::quote! {
        #[async_trait::async_trait]
        impl MindLogic for #name {
            const KIND: &'static str = stringify!(#name);

            fn new() -> Self {
                Self::default()
            }
        }
    }
    .into()
}
