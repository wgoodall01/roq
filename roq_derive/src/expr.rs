use roq_core::ast;
use syn::spanned::Spanned;

use crate::block::block_as_ast;

pub fn expr_as_ast(source: &syn::Expr) -> syn::Result<ast::Expr> {
    match source {
        // Traverse parenthesized expressions.
        syn::Expr::Paren(syn::ExprParen { expr, .. }) => expr_as_ast(expr),

        // Traverse block expressions.
        syn::Expr::Block(syn::ExprBlock { block, .. }) => block_as_ast(block),

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
            left, right, op, ..
        }) => {
            let lhs = expr_as_ast(left)?;
            let rhs = expr_as_ast(right)?;

            // Convert the name of the Rust operator to the name of a Coq function.
            // TODO: Make this work with non-nat types.
            let op_fn_name = match op {
                syn::BinOp::Add(_) => "plus",
                syn::BinOp::Sub(_) => "minus",
                syn::BinOp::Mul(_) => "mult",
                syn::BinOp::Div(_) => "Nat.div", // TODO: breaks with non-nat
                syn::BinOp::Rem(_) => "Nat.modulo", // TODO: breaks with non-nat
                syn::BinOp::And(_) => "andb",
                syn::BinOp::Or(_) => "orb",
                syn::BinOp::Shl(_) => "Nat.shiftl", // TODO: breaks with non-nat
                syn::BinOp::Shr(_) => "Nat.shiftr",
                syn::BinOp::Eq(_) => "Nat.eqb",
                syn::BinOp::Lt(_) => "Nat.ltb",
                syn::BinOp::Le(_) => "Nat.leb",

                // TODO: Find a function for Ne, Gt, Ge
                _ => return Err(syn::Error::new(op.span(), "Unsupported binary operator")),
            };

            Ok(ast::Expr::Apply {
                func: op_fn_name.into(),
                args: vec![lhs, rhs],
            })
        }

        // Match an if statement.
        syn::Expr::If(if_ex) => {
            let scrutinee = Box::new(expr_as_ast(&if_ex.cond)?);
            let mut cases = Vec::with_capacity(2);

            // Push the `then` branch
            cases.push(ast::MatchCase {
                pattern: ast::Pattern::Expr(ast::Expr::Bool(true)),
                body: block_as_ast(&if_ex.then_branch)?,
            });

            // Optionally push the else branch
            if let Some((_tok, else_expr)) = &if_ex.else_branch {
                cases.push(ast::MatchCase {
                    pattern: ast::Pattern::Expr(ast::Expr::Bool(false)),
                    body: expr_as_ast(else_expr)?,
                })
            } else {
                // Push unit
                cases.push(ast::MatchCase {
                    pattern: ast::Pattern::Expr(ast::Expr::Bool(false)),
                    body: ast::Expr::Tt,
                })
            }

            Ok(ast::Expr::Match { cases, scrutinee })
        }

        _ => Err(syn::Error::new(
            source.span(),
            format!("Unsupported expression type: {:?}", expr_name(source)),
        )),
    }
}

/// Give the name of an Expr.
fn expr_name(expr: &syn::Expr) -> &'static str {
    // This is literally the only way to do it.
    match expr {
        syn::Expr::Array(_) => "Array",
        syn::Expr::Assign(_) => "Assign",
        syn::Expr::Async(_) => "Async",
        syn::Expr::Await(_) => "Await",
        syn::Expr::Binary(_) => "Binary",
        syn::Expr::Block(_) => "Block",
        syn::Expr::Break(_) => "Break",
        syn::Expr::Call(_) => "Call",
        syn::Expr::Cast(_) => "Cast",
        syn::Expr::Closure(_) => "Closure",
        syn::Expr::Const(_) => "Const",
        syn::Expr::Continue(_) => "Continue",
        syn::Expr::Field(_) => "Field",
        syn::Expr::ForLoop(_) => "ForLoop",
        syn::Expr::Group(_) => "Group",
        syn::Expr::If(_) => "If",
        syn::Expr::Index(_) => "Index",
        syn::Expr::Infer(_) => "Infer",
        syn::Expr::Let(_) => "Let",
        syn::Expr::Lit(_) => "Lit",
        syn::Expr::Loop(_) => "Loop",
        syn::Expr::Macro(_) => "Macro",
        syn::Expr::Match(_) => "Match",
        syn::Expr::MethodCall(_) => "MethodCall",
        syn::Expr::Paren(_) => "Paren",
        syn::Expr::Path(_) => "Path",
        syn::Expr::Range(_) => "Range",
        syn::Expr::Reference(_) => "Reference",
        syn::Expr::Repeat(_) => "Repeat",
        syn::Expr::Return(_) => "Return",
        syn::Expr::Struct(_) => "Struct",
        syn::Expr::Try(_) => "Try",
        syn::Expr::TryBlock(_) => "TryBlock",
        syn::Expr::Tuple(_) => "Tuple",
        syn::Expr::Unary(_) => "Unary",
        syn::Expr::Unsafe(_) => "Unsafe",
        syn::Expr::Verbatim(_) => "Verbatim",
        syn::Expr::While(_) => "While",
        syn::Expr::Yield(_) => "Yield",
        _ => "[unknown]",
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
    fn test_binary_ops() {
        assert_snapshot!(expr("a + 1"), @r###"
        (plus a 1)
        "###);
        assert_snapshot!(expr("a - 1"), @r###"
        (minus a 1)
        "###);
        assert_snapshot!(expr("a * 1"), @r###"
        (mult a 1)
        "###);
        assert_snapshot!(expr("a / 1"), @r###"
        (Nat.div a 1)
        "###);
        assert_snapshot!(expr("a % 1"), @r###"
        (Nat.modulo a 1)
        "###);
        assert_snapshot!(expr("a && b"), @r###"
        (andb a b)
        "###);
        assert_snapshot!(expr("a || b"), @r###"
        (orb a b)
        "###);
        assert_snapshot!(expr("a << b"), @r###"
        (Nat.shiftl a b)
        "###);
        assert_snapshot!(expr("a >> b"), @r###"
        (Nat.shiftr a b)
        "###);
        assert_snapshot!(expr("a == b"), @r###"
        (Nat.eqb a b)
        "###);
    }

    #[test]
    fn test_if() {
        assert_snapshot!(
            expr("if a { b }"),
            @r###"
        match a with
        | true =>
        	b
        | false =>
        	tt
        end
        "###);
    }

    #[test]
    fn test_if_else() {
        assert_snapshot!(
            expr("if a { b } else { c }"),
            @r###"
        match a with
        | true =>
        	b
        | false =>
        	c
        end
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
