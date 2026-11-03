use syn::parse_quote;

pub fn ret_ty(ty: &syn::Type) -> syn::ReturnType {
    parse_quote!(-> #ty)
}

pub trait AsBlock {
    fn as_block(&self) -> syn::Block;
}

impl<T: quote::ToTokens> AsBlock for T {
    fn as_block(&self) -> syn::Block {
        parse_quote!({ #self })
    }
}

pub fn field(ident: &syn::Ident, ty: &syn::Type) -> syn::Field {
    syn::Field {
        attrs: vec![],
        vis: super::syn::visibility_default(),
        mutability: syn::FieldMutability::None,
        ident: Some(ident.clone()),
        colon_token: Some(Default::default()),
        ty: ty.clone()
    }
}
