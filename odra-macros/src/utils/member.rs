pub fn address() -> syn::Expr {
    syn::parse_quote!(self.address)
}

pub fn attached_value() -> syn::Expr {
    syn::parse_quote!(self.attached_value)
}

pub fn env() -> syn::Expr {
    syn::parse_quote!(self.env)
}

pub fn _self(ident: &syn::Ident) -> syn::Expr {
    syn::parse_quote!(self.#ident)
}
