use roq_core::ast;
use syn::spanned::Spanned;

pub fn type_as_ast(source: &syn::Type) -> syn::Result<ast::Ty> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    fn parse(input: &str) -> syn::Type {
        syn::parse_str(input).expect("Failed to parse source code")
    }

    fn test_as_ty(input: &str) -> ast::Ty {
        type_as_ast(&parse(input)).expect("Failed to convert to type")
    }

    #[test]
    fn test_u64() {
        assert_snapshot!(
            test_as_ty("u64"),
            @"nat"
        );
    }

    #[test]
    fn test_std_u64() {
        assert_snapshot!(
            test_as_ty("std::u64"),
            @"nat"
        );
    }

    #[test]
    fn test_bool() {
        assert_snapshot!(
            test_as_ty("bool"),
            @"nat"
        );
    }

    #[test]
    fn test_std_bool() {
        assert_snapshot!(
            test_as_ty("std::bool"),
            @"nat"
        );
    }
}
