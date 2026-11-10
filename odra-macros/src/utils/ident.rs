use quote::format_ident;

// The generated code binds the arguments of an entry point and the fields of a module, a type or
// an event to locals named exactly like them. Its own locals, parameters and fields that share a
// scope with these are prefixed with `__odra_`, so that no user name can shadow them or be
// shadowed by them.

pub fn named_args() -> syn::Ident {
    format_ident!("__odra_named_args")
}

pub fn contract_env() -> syn::Ident {
    format_ident!("__odra_contract_env")
}

pub fn result() -> syn::Ident {
    format_ident!("__odra_result")
}

pub fn call_def() -> syn::Ident {
    format_ident!("__odra_call_def")
}

/// The env of a module: its hidden field, the parameter of `Module::new` and of the generated
/// `execute_*` functions.
pub fn prefixed_env() -> syn::Ident {
    format_ident!("__odra_env")
}

pub fn exec_env() -> syn::Ident {
    format_ident!("__odra_exec_env")
}

pub fn contract() -> syn::Ident {
    format_ident!("__odra_contract")
}

pub fn env_rc() -> syn::Ident {
    format_ident!("__odra_env_rc")
}

pub fn bytes() -> syn::Ident {
    format_ident!("__odra_bytes")
}
