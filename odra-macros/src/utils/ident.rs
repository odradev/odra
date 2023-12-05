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

pub fn exec_env() -> syn::Ident {
    format_ident!("exec_env")
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

pub fn entry_points() -> syn::Ident {
    format_ident!("entry_points")
}

pub fn add_entry_point() -> syn::Ident {
    format_ident!("add_entry_point")
}

pub fn new() -> syn::Ident {
    format_ident!("new")
}

pub fn schemas() -> syn::Ident {
    format_ident!("schemas")
}

pub fn contract() -> syn::Ident {
    format_ident!("contract")
}

pub fn env_rc() -> syn::Ident {
    format_ident!("env_rc")
}
