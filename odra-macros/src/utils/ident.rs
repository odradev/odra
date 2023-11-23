use quote::format_ident;

pub fn named_args() -> syn::Ident {
    format_ident!("named_args")
}

pub fn contract_env() -> syn::Ident {
    format_ident!("contract_env")
}

pub fn result() -> syn::Ident {
    format_ident!("result")
}

pub fn call_def() -> syn::Ident {
    format_ident!("call_def")
}

pub fn caller() -> syn::Ident {
    format_ident!("caller")
}

pub fn env() -> syn::Ident {
    format_ident!("env")
}

pub fn init() -> syn::Ident {
    format_ident!("init")
}

pub fn address() -> syn::Ident {
    format_ident!("address")
}

pub fn attached_value() -> syn::Ident {
    format_ident!("attached_value")
}
