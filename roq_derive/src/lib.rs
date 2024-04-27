use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use roq_core::ast;
use syn::spanned::Spanned;

/// Generate a Coq `Definition` statement from a Rust function.
#[proc_macro_attribute]
pub fn definition(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = TokenStream2::from(item);
    let input: syn::ItemFn = match syn::parse2(item.clone()) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    // Convert the function to a Coq `Definition` AST node.
    let definition = match fn_as_definition(&input) {
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

fn fn_as_definition(source: &syn::ItemFn) -> syn::Result<ast::Definition> {
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

fn expr_as_v(source: &syn::Expr) -> syn::Result<ast::Expr> {
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

fn type_as_v(source: &syn::Type) -> syn::Result<ast::Ty> {
    match source {
        syn::Type::Path(ty) => {
            let segments_str = ty
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>();
            let segments_refs = segments_str.iter().map(|s| s.as_str()).collect::<Vec<_>>();

            match segments_refs[..] {
                ["u64"] | ["std", "u64"] => Ok(ast::Ty::Nat),
                ["bool"] | ["std", "bool"] => Ok(ast::Ty::Nat),
                _ => Err(syn::Error::new(ty.span(), "unsupported type")),
            }
        }
        _ => Err(syn::Error::new(
            source.span(),
            "expected a path type (e.g. `std::u64` or `bool`)",
        )),
    }
}
