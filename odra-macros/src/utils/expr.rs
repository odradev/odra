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

pub fn new_type(ty: &syn::Type, env_ident: &syn::Ident, idx: u8) -> syn::Expr {
    let rc = rc_clone(env_ident);
    parse_quote!(#ty::new(#rc, #idx))
}

fn rc_clone(ident: &syn::Ident) -> syn::Expr {
    parse_quote!(Rc::clone(&#ident))
}

pub fn new_entry_points() -> syn::Expr {
    parse_quote!(odra::casper_types::EntryPoints::new())
}

pub fn entry_point_contract() -> syn::Expr {
    parse_quote!(odra::casper_types::EntryPointType::Contract)
}

pub fn entry_point_public() -> syn::Expr {
    parse_quote!(odra::casper_types::EntryPointAccess::Public)
}

pub fn entry_point_group(name: &str) -> syn::Expr {
    parse_quote!(odra::casper_types::EntryPointAccess::Groups(vec![odra::casper_types::Group::new(#name)]))
}

pub fn new_parameter(name: String, ty: syn::Type) -> syn::Expr {
    let cl_type = as_cl_type(&ty);
    parse_quote!(odra::casper_types::Parameter::new(#name, #cl_type))
}

pub fn as_cl_type(ty: &syn::Type) -> syn::Expr {
    let ty = match ty {
        syn::Type::Path(type_path) => {
            let mut segments: syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]> = type_path.path.segments.clone();
            // the syntax <Option<U256> as odra::casper_types::CLTyped>::cl_type() is invalid
            // it should be <Option::<U256> as odra::casper_types::CLTyped>::cl_type()
            segments
                .first_mut()
                .map(|ps| if let syn::PathArguments::AngleBracketed(ab) = &ps.arguments {
                    let generic_arg: syn::AngleBracketedGenericArguments = parse_quote!(::#ab);
                    ps.arguments = syn::PathArguments::AngleBracketed(generic_arg);
                });
            syn::Type::Path(syn::TypePath { 
                path: syn::Path { leading_colon: None, segments },
                ..type_path.clone()
            })
        },
        _ => ty.clone(),
    };
    parse_quote!(<#ty as odra::casper_types::CLTyped>::cl_type())
}

pub fn unit_cl_type() -> syn::Expr {
    parse_quote!(<() as odra::casper_types::CLTyped>::cl_type())
}