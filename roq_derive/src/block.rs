use roq_core::ast;
use syn::spanned::Spanned;

use crate::expr::expr_as_ast;

pub fn block_as_ast(block: &syn::Block) -> syn::Result<ast::Expr> {
    // Make sure the function consists of supported statements.
    let stmts = block.stmts.as_slice();

    // Empty blocks evaluate to the unit type.
    if stmts.is_empty() {
        return Ok(ast::Expr::Tt);
    }

    // Match the last statement, the implicit return value.
    let last_stmt = stmts.last().unwrap();
    let mut seq_stmt = match last_stmt {
        syn::Stmt::Expr(expr, _) => expr_as_ast(expr)?,
        _ => {
            return Err(syn::Error::new(
                last_stmt.span(),
                "Expected function to end with an expr",
            ))
        }
    };

    // Iterate backwards through the statements, building up the AST.
    for stmt in stmts.iter().rev() {
        match stmt {
            // Ignore useless expressions.
            syn::Stmt::Expr(_expr, _) => {}

            // Convert local variable declarations to `LetIn` expressions.
            syn::Stmt::Local(local) => {
                let Some(local_init) = &local.init else {
                    return Err(syn::Error::new(
                        local.span(),
                        "expected local variable to be initialized",
                    ));
                };
                let local_init_expr = expr_as_ast(&local_init.expr)?;

                let ident = match &local.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => {
                        return Err(syn::Error::new(
                            local.pat.span(),
                            "expected a single identifier, not a pattern, in local variable",
                        ))
                    }
                };

                seq_stmt = roq_core::ast::Expr::LetIn {
                    ident,
                    value: Box::new(local_init_expr),
                    child: Box::new(seq_stmt),
                };
            }
            _ => {}
        }
    }

    Ok(seq_stmt)
}
