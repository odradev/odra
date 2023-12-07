pub fn field(ident: &syn::Ident, ty: &syn::Type) -> syn::Field {
    syn::Field {
        attrs: vec![],
        vis: super::syn::visibility_private(),
        mutability: syn::FieldMutability::None,
        ident: Some(ident.clone()),
        colon_token: Some(Default::default()),
        ty: ty.clone()
    }
}
