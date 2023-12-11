pub fn address() -> syn::ExprField {
    member(super::ident::address())
}

pub fn attached_value() -> syn::ExprField {
    member(super::ident::attached_value())
}

pub fn env() -> syn::ExprField {
    member(super::ident::env())
}

pub fn underscored_env() -> syn::ExprField {
    member(super::ident::underscored_env())
}

fn member(ident: syn::Ident) -> syn::ExprField {
    syn::parse_quote!(self.#ident)
}
