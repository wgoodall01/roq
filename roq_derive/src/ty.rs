use roq_core::ast;
use syn::spanned::Spanned;

pub fn type_as_v(source: &syn::Type) -> syn::Result<ast::Ty> {
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
