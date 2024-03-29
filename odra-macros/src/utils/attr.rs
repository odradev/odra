use syn::parse_quote;

pub fn not_wasm32() -> syn::Attribute {
    parse_quote!(#[cfg(not(target_arch = "wasm32"))])
}

pub fn wasm32() -> syn::Attribute {
    parse_quote!(#[cfg(target_arch = "wasm32")])
}

pub fn odra_module(name: &str) -> syn::Attribute {
    parse_quote!(#[cfg(odra_module = #name)])
}

pub fn no_mangle() -> syn::Attribute {
    parse_quote!(#[no_mangle])
}

pub fn inline() -> syn::Attribute {
    parse_quote!(#[inline])
}

pub fn automatically_derived() -> syn::Attribute {
    parse_quote!(#[automatically_derived])
}

pub fn derive_into_runtime_args() -> syn::Attribute {
    parse_quote!(#[derive(odra::IntoRuntimeArgs)])
}

pub fn init_args_docs(name: String) -> syn::Attribute {
    let name = format!(" [{}] contract constructor arguments.", name);
    parse_quote!(#[doc = #name])
}

pub fn missing_docs() -> syn::Attribute {
    parse_quote!(#[allow(missing_docs)])
}

pub fn common_derive_attr() -> syn::Attribute {
    parse_quote!(#[derive(Clone, PartialEq, Eq, Debug)])
}
