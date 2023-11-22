pub fn address() -> syn::ExprField {
    syn::parse_quote!(self.address)
}

pub fn attached_value() -> syn::ExprField {
    syn::parse_quote!(self.attached_value)
}

pub fn env() -> syn::ExprField {
    syn::parse_quote!(self.env)
}
