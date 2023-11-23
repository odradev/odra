use syn::parse_quote;

pub fn new_runtime_args() -> syn::Expr {
    parse_quote!(odra::RuntimeArgs::new())
}

pub fn u512_zero() -> syn::Expr {
    parse_quote!(odra::U512::zero())
}

pub fn parse_bytes(data_ident: &syn::Ident) -> syn::Expr {
    parse_quote!(odra::ToBytes::to_bytes(&#data_ident).map(Into::into).unwrap())
}
