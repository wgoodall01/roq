use roq_core::ast;
use syn::spanned::Spanned;

use crate::{expr::expr_as_v, ty::type_as_v};

pub fn fn_as_definition(source: &syn::ItemFn) -> syn::Result<ast::Definition> {
    let name = source.sig.ident.to_string();

    // Map the return type, which is mandatory.
    let ret = match &source.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new(
                source.sig.output.span(),
                "expected a return type",
            ))
        }
        syn::ReturnType::Type(_, ty) => type_as_v(ty)?,
    };

    // Map each of the arguments.
    let mut args = vec![];
    for arg in &source.sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new(
                    arg.span(),
                    "can't generate Coq `Definition` for function which takes `self`",
                ))
            }

            syn::FnArg::Typed(pat) => {
                let name = match &*pat.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => {
                        return Err(syn::Error::new(
                            pat.pat.span(),
                            "expected a single identifier, not a pattern, in function argument",
                        ))
                    }
                };
                let ty = type_as_v(&pat.ty)?;
                args.push(ast::Binder { name, ty });
            }
        }
    }

    // Make sure the function consists of a single statement.
    let statement = match source.block.stmts.as_slice() {
        [stmt] => stmt,
        [] => {
            return Err(syn::Error::new(
                source.block.span(),
                "expected at least one statement in block",
            ))
        }

        // TODO: this is a really dumb restriction.
        [..] => {
            return Err(syn::Error::new(
                source.block.span(),
                "expected exactly one statement in function body",
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

    // Parse binary addition to start out.
    let body = match expr {
        // Binary addition
        syn::Expr::Binary(syn::ExprBinary {
            left,
            right,
            op: syn::BinOp::Add(_),
            ..
        }) => {
            let lhs = expr_as_v(&left)?;
            let rhs = expr_as_v(&right)?;
            ast::Expr::Apply {
                func: "plus".into(),
                args: vec![lhs, rhs],
            }
        }
        _ => panic!("unsupported expression"),
    };

    Ok(ast::Definition {
        name,
        args,
        ret,
        body,
    })
}
