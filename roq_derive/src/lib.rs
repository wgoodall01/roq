use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod expr;
mod func;
mod ty;

/// Generate a Coq `Definition` statement from a Rust function.
#[proc_macro_attribute]
pub fn definition(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = TokenStream2::from(item);
    let input: syn::ItemFn = match syn::parse2(item.clone()) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    // Convert the function to a Coq `Definition` AST node.
    let definition = match func::func_as_ast(&input) {
        Ok(definition) => definition,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    // Serialize this to Rust literal tokens.
    let definition_str = uneval::to_string(definition)
        .expect("Failed to serialize Coq AST to Rust literal expressions");
    let definition_tokens: TokenStream2 = definition_str
        .parse()
        .expect("Serialized AST is not a valid Rust literal expression");

    // Emit the original, unmodified function, plus a module named `$fn_name::roq` containing an
    // `as_definition` function that returns the Coq `Definition`.
    let fn_name = input.sig.ident;

    TokenStream::from(quote! {
        #item
        #[doc(hidden)]
        pub mod #fn_name {
            pub mod roq {
                pub fn definition() -> roq_core::ast::Definition {
                    use ::roq_core::ast::*;
                    #definition_tokens
                }
                pub fn vernacular() -> roq_core::ast::Vernacular {
                    let defn = definition();
                    ::roq_core::ast::Statement::Definition(defn).into()
                }
            }
        }
    })
}
