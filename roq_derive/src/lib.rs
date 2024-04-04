use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Result;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemFn};

/// Generate a Coq `Definition` statement from a Rust function.
#[proc_macro_attribute]
pub fn definition(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let name = input.sig.ident.to_string();
    let ret = match &input.sig.output {
        syn::ReturnType::Default => panic!("`Definition` functions must have a return type"),
        syn::ReturnType::Type(_, ty) => CoqType::try_from(&**ty).unwrap(),
    };

    let args = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => panic!("`Definition` functions cannot take `self`"),
            syn::FnArg::Typed(pat) => {
                let name = match &*pat.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => panic!("unsupported argument pattern"),
                };
                let ty = CoqType::try_from(&*pat.ty).unwrap();
                Some(Arg { name, ty })
            }
        })
        .collect::<Vec<_>>();

    // Make sure the function has one statement
    let statement = match input.block.stmts.as_slice() {
        [stmt] => stmt,
        [] => panic!("`Definition` functions must have a body"),
        [..] => panic!("`Definition` functions must have a single statement"),
    };

    let expr: syn::Expr = match statement {
        syn::Stmt::Expr(expr, _) => expr.clone(),
        _ => panic!("`Definition` functions must have a single expression statement"),
    };

    // Parse binary addition
    let body = match expr {
        syn::Expr::Binary(syn::ExprBinary {
            left,
            right,
            op: syn::BinOp::Add(_),
            ..
        }) => {
            let left = match *left {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(int),
                    ..
                }) => CoqExpr::Nat(int.base10_parse().unwrap()),
                syn::Expr::Path(syn::ExprPath {
                    path: syn::Path { segments, .. },
                    ..
                }) => CoqExpr::Var(segments.first().unwrap().ident.to_string()),
                _ => panic!("unsupported expression"),
            };

            let right = match *right {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(int),
                    ..
                }) => CoqExpr::Nat(int.base10_parse().unwrap()),
                syn::Expr::Path(syn::ExprPath {
                    path: syn::Path { segments, .. },
                    ..
                }) => CoqExpr::Var(segments.first().unwrap().ident.to_string()),
                _ => panic!("unsupported expression"),
            };

            CoqExpr::nat_add(left, right)
        }
        _ => panic!("unsupported expression"),
    };

    let coq_defn = CoqDefinition {
        name,
        args,
        ret,
        body,
    };
    println!("{coq_defn:#?}");
    println!("{coq_defn:#}");

    TokenStream::from(quote!(#input))
}

#[derive(Debug, Clone)]
struct CoqDefinition {
    name: String,
    args: Vec<Arg>,
    ret: CoqType,
    body: CoqExpr,
}

impl std::fmt::Display for CoqDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Definition {}", self.name)?;
        for arg in &self.args {
            write!(f, " ({}: {})", arg.name, arg.ty)?;
        }
        write!(f, " : {} := ", self.ret)?;
        write!(f, "({})", self.body)?;
        write!(f, ".")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Arg {
    name: String,
    ty: CoqType,
}

#[derive(Debug, Clone)]
enum CoqType {
    Nat,
}

impl std::fmt::Display for CoqType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CoqType::Nat => write!(f, "nat"),
        }
    }
}

impl TryFrom<&syn::Type> for CoqType {
    type Error = syn::Error;

    fn try_from(ty: &syn::Type) -> Result<Self> {
        match ty {
            syn::Type::Path(ty) => {
                let seg = ty.path.segments.first().ok_or_else(|| {
                    syn::Error::new(ty.path.span(), "expected a single path segment")
                })?;
                let ident = seg.ident.to_string();
                match ident.as_str() {
                    "u64" => Ok(CoqType::Nat),
                    _ => Err(syn::Error::new(ty.span(), "unsupported type")),
                }
            }
            _ => Err(syn::Error::new(ty.span(), "unsupported type")),
        }
    }
}

#[derive(Debug, Clone)]
enum CoqExpr {
    /// Apply a function.
    Apply { func: String, args: Vec<CoqExpr> },

    /// A variable name.
    Var(String),

    /// A `nat` literal.
    Nat(u64),
}

impl CoqExpr {
    /// Add two `nat`s.
    fn nat_add(a: CoqExpr, b: CoqExpr) -> CoqExpr {
        CoqExpr::Apply {
            func: "Nat.add".into(),
            args: vec![a, b],
        }
    }
}

impl std::fmt::Display for CoqExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CoqExpr::Apply { func, args } => {
                write!(f, "(")?;
                write!(f, "{}", func)?;
                if !args.is_empty() {
                    write!(
                        f,
                        " {}",
                        args.iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    )?;
                }
                write!(f, ")")?;
                Ok(())
            }
            CoqExpr::Var(name) => write!(f, "{}", name),
            CoqExpr::Nat(n) => write!(f, "{}", n),
        }
    }
}
