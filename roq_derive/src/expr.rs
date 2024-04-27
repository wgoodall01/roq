use roq_core::ast;
use syn::spanned::Spanned;

pub fn expr_as_v(source: &syn::Expr) -> syn::Result<ast::Expr> {
    match source {
        // Match integer literals.
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) => Ok(ast::Expr::Nat(int.base10_parse().unwrap())),

        // Match boolean literals.
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(boolean),
            ..
        }) => Ok(ast::Expr::Bool(boolean.value)),

        // Match variables.
        syn::Expr::Path(syn::ExprPath {
            path: syn::Path { segments, .. },
            ..
        }) => match segments.iter().collect::<Vec<_>>().as_slice() {
            [segment] => {
                let ident = &segment.ident;
                Ok(ast::Expr::Var(ident.to_string()))
            }
            _ => Err(syn::Error::new(
                segments.first().unwrap().ident.span(),
                "expected a single path segment",
            )),
        },

        _ => Err(syn::Error::new(
            source.span(),
            "expected an integer literal, boolean literal, or a variable",
        )),
    }
}
