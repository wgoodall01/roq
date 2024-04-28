use roq_core::ast;
use syn::spanned::Spanned;

use crate::{expr::expr_as_ast, ty::type_as_ast};

pub fn func_as_ast(source: &syn::ItemFn) -> syn::Result<ast::Definition> {
    let name = source.sig.ident.to_string();

    // Map the return type, which is mandatory.
    let ret = match &source.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new(
                source.sig.output.span(),
                "expected a return type",
            ))
        }
        syn::ReturnType::Type(_, ty) => type_as_ast(ty)?,
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
                let ty = type_as_ast(&pat.ty)?;
                args.push(ast::Binder { name, ty });
            }
        }
    }

    // Make sure the function consists of supported statements.
    let stmts = source.block.stmts.as_slice();

    if stmts.is_empty() {
        return Err(syn::Error::new(
            source.block.span(),
            "expected at least one statement in block",
        ));
    }

    let mut seq_stmt = match &stmts[stmts.len() - 1] {
        syn::Stmt::Expr(expr, _) => expr_as_ast(expr)?,
        _ => {
            return Err(syn::Error::new(
                source.block.span(),
                "Expected function to end with an expr",
            ))
        }
    };

    for stmt in stmts.iter().rev() {
        match stmt {
            syn::Stmt::Expr(_expr, _) => {}
            syn::Stmt::Local(local) => {
                let Some(local_init) = &local.init else {
                    return Err(syn::Error::new(
                        local.span(),
                        "expected a local variable to be initialized",
                    ));
                };
                let local_init_expr = expr_as_ast(&local_init.expr)?;

                seq_stmt =
                    roq_core::ast::Expr::LetIn {
                        ident: match &local.pat {
                            syn::Pat::Ident(ident) => ident.ident.to_string(),
                            _ => return Err(syn::Error::new(
                                local.pat.span(),
                                "expected a single identifier, not a pattern, in local variable",
                            )),
                        },
                        value: Box::new(local_init_expr),
                        child: Box::new(seq_stmt),
                    };
            }
            _ => {}
        }
    }

    // Parse the body of the statement.
    let body = seq_stmt;

    Ok(ast::Definition {
        name,
        args,
        ret,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    fn parse(input: &str) -> syn::ItemFn {
        syn::parse_str(input).expect("Failed to parse source code")
    }

    fn test_as_def(input: &str) -> ast::Definition {
        func_as_ast(&parse(input)).expect("Failed to convert function to definition")
    }

    #[test]
    fn test_binary_add() {
        assert_snapshot!(
            test_as_def(r#"
                fn add(a: u64, b: u64) -> u64 {
                    a + b
                }
            "#),
            @r###"
        Definition add (a: nat) (b: nat) : nat :=
        	(plus a b)
        .
        "###
        );
    }

    #[test]
    fn test_add_one() {
        assert_snapshot!(
            test_as_def(r#"
                fn add(a: u64) -> u64 {
                    a + 1
                }
            "#),
            @r###"
        Definition add (a: nat) : nat :=
        	(plus a 1)
        .
        "###
        );
    }

    #[test]
    fn test_let_x_add() {
        assert_snapshot!(
            test_as_def(r#"
                fn add(a: u64) -> u64 {
                    let x = 1;
                    a + x
                }
            "#),
            @r###"
        Definition add (a: nat) : nat :=
        	let x := 1 in
        	(plus a x)
        .
        "###
        );
    }

    #[test]
    fn test_let_x_y_add() {
        assert_snapshot!(
            test_as_def(r#"
                fn add(a: u64) -> u64 {
                    let x = 1;
                    let y = 2;
                    x + y
                }
            "#),
            @r###"
        Definition add (a: nat) : nat :=
        	let x := 1 in
        	let y := 2 in
        	(plus x y)
        .
        "###
        );
    }
}
