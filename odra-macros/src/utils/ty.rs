use syn::parse_quote;

pub fn address() -> syn::Type {
    parse_quote!(odra::Address)
}

pub fn contract_env() -> syn::Type {
    parse_quote!(odra::ContractEnv)
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
