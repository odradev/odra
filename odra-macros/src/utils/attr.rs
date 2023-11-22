use syn::parse_quote;

pub fn not_wasm32() -> syn::Attribute {
    parse_quote!(#[cfg(not(target_arch = "wasm32"))])
}
