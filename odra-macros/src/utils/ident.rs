use quote::format_ident;

// The generated code binds each argument of an entry point to a local named exactly like the
// argument. Its own locals (and parameters) that share a scope with these are prefixed with
// `__odra_`, so that no argument name can shadow them or be shadowed by them.

pub fn named_args() -> syn::Ident {
    format_ident!("__odra_named_args")
}

pub fn contract_env() -> syn::Ident {
    format_ident!("__odra_contract_env")
}

pub fn result() -> syn::Ident {
    format_ident!("result")
}

pub fn call_def() -> syn::Ident {
    format_ident!("__odra_call_def")
}

pub fn env() -> syn::Ident {
    format_ident!("env")
}

/// The `ContractEnv` parameter of a generated `execute_*` function.
pub fn exec_fn_env() -> syn::Ident {
    format_ident!("__odra_env")
}

/// The value returned by an entry point in the code that calls it.
pub fn exec_result() -> syn::Ident {
    format_ident!("__odra_result")
}

pub fn underscored_env() -> syn::Ident {
    format_ident!("__env")
}

pub fn exec_env() -> syn::Ident {
    format_ident!("__odra_exec_env")
}

pub fn epc() -> syn::Ident {
    format_ident!("entry_points_caller")
}

pub fn address() -> syn::Ident {
    format_ident!("address")
}

pub fn contract_address() -> syn::Ident {
    format_ident!("contract_address")
}

pub fn attached_value() -> syn::Ident {
    format_ident!("attached_value")
}

pub fn entry_points() -> syn::Ident {
    format_ident!("entry_points")
}

pub fn child_contract_entry_points() -> syn::Ident {
    format_ident!("child_contract_entry_points")
}

pub fn add_entry_point() -> syn::Ident {
    format_ident!("add")
}

pub fn schemas() -> syn::Ident {
    format_ident!("schemas")
}

pub fn contract() -> syn::Ident {
    format_ident!("__odra_contract")
}

pub fn env_rc() -> syn::Ident {
    format_ident!("__odra_env_rc")
}

pub fn events() -> syn::Ident {
    format_ident!("events")
}
pub fn event_schemas() -> syn::Ident {
    format_ident!("event_schemas")
}

pub fn module_schema() -> syn::Ident {
    format_ident!("module_schema")
}

pub fn entrypoints() -> syn::Ident {
    format_ident!("entrypoints")
}

pub fn ident() -> syn::Ident {
    format_ident!("ident")
}

pub fn bytes() -> syn::Ident {
    format_ident!("bytes")
}

pub fn from_bytes() -> syn::Ident {
    format_ident!("from_bytes")
}

pub fn to_bytes() -> syn::Ident {
    format_ident!("to_bytes")
}

pub fn serialized_length() -> syn::Ident {
    format_ident!("serialized_length")
}

pub fn cl_type() -> syn::Ident {
    format_ident!("cl_type")
}

pub fn from() -> syn::Ident {
    format_ident!("from")
}

pub fn error() -> syn::Ident {
    format_ident!("error")
}
