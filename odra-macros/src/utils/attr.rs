use syn::parse_quote;

pub fn odra_module(name: &str) -> syn::Attribute {
    odra_module_of_crate(&current_crate_name(), name)
}

fn odra_module_of_crate(crate_name: &str, name: &str) -> syn::Attribute {
    let qualified = format!("{}::{}", crate_name, name);
    parse_quote!(#[cfg(any(odra_module = #name, odra_module = #qualified))])
}

/// The name of the crate the macro is currently expanding in, with `-` already replaced by `_`
/// by cargo.
///
/// `CARGO_CRATE_NAME` is the only environment variable the macros read: proc macros run inside
/// the `rustc` invocation cargo sets up, so it is always present in a cargo build and tells the
/// macro which crate defines the module. That lets the generated wasm parts be gated by a
/// crate-qualified `odra_module` value, so two crates defining a module of the same name do not
/// both compile their entry points into a single wasm.
pub(crate) fn current_crate_name() -> String {
    std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "unknown_crate".to_string())
}

#[cfg(test)]
mod tests;
