use syn::parse_quote;

pub fn new_runtime_args() -> syn::Expr {
    let ty = super::ty::runtime_args();
    parse_quote!(#ty::new())
}

pub fn u512_zero() -> syn::Expr {
    let ty = super::ty::u512();
    parse_quote!(#ty::zero())
}

pub fn parse_bytes(data_ident: &syn::Ident) -> syn::Expr {
    parse_quote!(odra::ToBytes::to_bytes(&#data_ident).map(Into::into).unwrap())
}

pub fn new_type(ty: &syn::Type, env_ident: &syn::Ident, idx: u8) -> syn::Expr {
    let rc = rc_clone(env_ident);
    parse_quote!(#ty::new(#rc, #idx))
}

fn rc_clone(ident: &syn::Ident) -> syn::Expr {
    parse_quote!(Rc::clone(&#ident))
}

pub fn new_entry_points() -> syn::Expr {
    let ty = super::ty::entry_points();
    parse_quote!(#ty::new())
}

pub fn entry_point_contract() -> syn::Expr {
    let ty = super::ty::entry_point_type();
    parse_quote!(#ty::Contract)
}

pub fn entry_point_public() -> syn::Expr {
    let ty = super::ty::entry_point_access();
    parse_quote!(#ty::Public)
}

pub fn entry_point_group(name: &str) -> syn::Expr {
    let ty = super::ty::entry_point_access();
    let ty_group = super::ty::group();
    parse_quote!(#ty::Groups(vec![#ty_group::new(#name)]))
}

pub fn new_parameter(name: String, ty: syn::Type) -> syn::Expr {
    let ty_param = super::ty::parameter();
    let cl_type = as_cl_type(&ty);
    parse_quote!(#ty_param::new(#name, #cl_type))
}

pub fn as_cl_type(ty: &syn::Type) -> syn::Expr {
    let ty = super::syn::unreferenced_ty(ty);
    let ty_cl_typed = super::ty::cl_typed();
    let ty = super::syn::as_casted_ty_stream(&ty, ty_cl_typed);
    parse_quote!(#ty::cl_type())
}

pub fn unit_cl_type() -> syn::Expr {
    let ty_cl_typed = super::ty::cl_typed();
    parse_quote!(<() as #ty_cl_typed>::cl_type())
}

pub fn new_schemas() -> syn::Expr {
    let ty = super::ty::schemas();
    parse_quote!(#ty::new())
}

pub fn new_wasm_contract_env() -> syn::Expr {
    parse_quote!(odra::odra_casper_wasm_env::WasmContractEnv::new_env())
}

pub fn into_event(ty: &syn::Type) -> syn::Expr {
    parse_quote!(<#ty as odra::contract_def::IntoEvent>::into_event())
}

pub fn events(ty: &syn::Type) -> syn::Expr {
    let has_events_ty = super::ty::has_events();
    parse_quote!(<#ty as #has_events_ty>::events())
}
pub fn new_blueprint(ident: &syn::Ident) -> syn::Expr {
    let ty = super::ty::contract_blueprint();
    parse_quote!(#ty::new::<#ident>())
}

pub fn string_from(string: String) -> syn::Expr {
    parse_quote!(String::from(#string))
}

pub trait IntoExpr {
    fn into_expr(self) -> syn::Expr;
}

impl IntoExpr for syn::Ident {
    fn into_expr(self) -> syn::Expr {
        parse_quote!(#self)
    }
}
