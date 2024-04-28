use roq_core::ast;
use syn::spanned::Spanned;

use crate::expr::expr_as_ast;

pub fn block_as_ast(source: &syn::Block) -> syn::Result<ast::Expr> {
    // TODO: bring in Flynt's work on blocks.

    // Make sure the blockj consists of a single statement.
    let statement = match source.stmts.as_slice() {
        [stmt] => stmt,

        // For empty blocks, return unit.
        [] => {
            return Ok(ast::Expr::Tt);
        }

        // TODO: this is a really dumb restriction.
        [..] => {
            return Err(syn::Error::new(
                source.span(),
                "expected exactly one statement in block",
            ))
        }
    };

    // Make sure that statement is an expression.
    let expr: syn::Expr = match statement {
        syn::Stmt::Expr(expr, _) => expr.clone(),
        _ => {
            return Err(syn::Error::new(
                statement.span(),
                "expected statement to be an expression",
            ))
        }
    };

    // Parse the body of the statement.
    expr_as_ast(&expr)
}
