use syn::parse_quote;


pub fn address() -> syn::Type {
    parse_quote!(odra::Address)
}

pub fn contract_env() -> syn::Type {
    parse_quote!(odra::ContractEnv)
}

pub fn rc_contract_env() -> syn::Type {
    parse_quote!(Rc<odra::ContractEnv>)
}

pub fn is_rc_contract_env(ty: &syn::Type) -> bool {
    let rc: syn::Type = parse_quote!(Rc<ContractEnv>);
    test_utils::eq(ty, &rc) || test_utils::eq(ty, &rc_contract_env())
}

pub fn from_bytes() -> syn::Type {
    parse_quote!(odra::FromBytes)
}

pub fn event_instance() -> syn::Type {
    parse_quote!(odra::casper_event_standard::EventInstance)
}

pub fn event_error() -> syn::Type {
    parse_quote!(odra::event::EventError)
}

pub fn u512() -> syn::Type {
    parse_quote!(odra::U512)
}

pub fn host_env() -> syn::Type {
    parse_quote!(odra::HostEnv)
}

pub fn call_def() -> syn::Type {
    parse_quote!(odra::CallDef)
}

pub fn entry_points_caller() -> syn::Type {
    parse_quote!(odra::EntryPointsCaller)
}

pub fn contract_call_result() -> syn::Type {
    parse_quote!(odra::ContractCallResult)
}

pub fn odra_error() -> syn::Type {
    parse_quote!(odra::OdraError)
}

pub fn module_wrapper() -> syn::Type {
    parse_quote!(odra::ModuleWrapper)
}

pub fn module() -> syn::Type {
    parse_quote!(odra::module::Module)
}

pub fn variable() -> syn::Type {
    parse_quote!(odra::Variable)
}

pub fn mapping() -> syn::Type {
    parse_quote!(odra::Mapping)
}

pub fn super_path(ident: syn::Ident) -> syn::Type {
    parse_quote!(super::#ident)
}
