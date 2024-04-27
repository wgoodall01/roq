use roq_core::ast;
use syn::spanned::Spanned;

pub fn expr_as_ast(source: &syn::Expr) -> syn::Result<ast::Expr> {
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

        // Match binary arithmetic.
        syn::Expr::Binary(syn::ExprBinary {
            left,
            right,
            op: syn::BinOp::Add(_),
            ..
        }) => {
            let lhs = expr_as_ast(left)?;
            let rhs = expr_as_ast(right)?;
            Ok(ast::Expr::Apply {
                func: "plus".into(),
                args: vec![lhs, rhs],
            })
        }

        _ => Err(syn::Error::new(
            source.span(),
            "Unsupported expression type",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    fn parse(input: &str) -> syn::Expr {
        syn::parse_str(input).expect("Failed to parse source code")
    }

    fn expr(input: &str) -> ast::Expr {
        expr_as_ast(&parse(input)).expect("Failed to convert to expr")
    }

    #[test]
    fn test_binary_add() {
        assert_snapshot!(
            expr("a + 1"), 
            @r###"
        (plus a 1)
        "###);
    }

    #[test]
    fn test_ternary_add() {
        assert_snapshot!(
            expr("1 + 2 + 3"), 
            @r###"
        (plus (plus 1 2)
         3)
        "###);
    }
}
