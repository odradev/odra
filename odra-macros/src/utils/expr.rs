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

pub fn new_variable(env_ident: &syn::Ident, idx: u8) -> syn::Expr {
    let ty = super::ty::variable();
    new_type(&ty, env_ident, idx)
}

pub fn new_mapping(env_ident: &syn::Ident, idx: u8) -> syn::Expr {
    let ty = super::ty::mapping();
    new_type(&ty, env_ident, idx)
}

pub fn new_module_wrapper(env_ident: &syn::Ident, idx: u8) -> syn::Expr {
    let ty = super::ty::module_wrapper();
    new_type(&ty, env_ident, idx)
}

pub fn new_type(ty: &syn::Type, env_ident: &syn::Ident, idx: u8) -> syn::Expr {
    let rc = rc_clone(env_ident);
    dbg!(quote::quote!(#ty::new(#rc, #idx)).to_string());
    parse_quote!(#ty::new(#rc, #idx))
}

fn rc_clone(ident: &syn::Ident) -> syn::Expr {
    parse_quote!(Rc::clone(&#ident))
}
