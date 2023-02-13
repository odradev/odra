use std::str::FromStr;

use proc_macro2::TokenStream;
use syn::Path;

pub fn fqn_to_path(fqn: &String) -> Path {
    let tokens = TokenStream::from_str(fqn).expect("fqn should be a valid token stream");
    syn::parse2::<syn::Path>(tokens).expect("Couldn't parse token stream")
}

#[cfg(test)]
mod test {
    use super::fqn_to_path;

    #[test]
    fn parsing_fqn() {
        let fqn = String::from("full::contract::path::Contract");

        let path: syn::Path = syn::parse_quote! {
            full::contract::path::Contract
        };
        assert_eq!(path, fqn_to_path(&fqn));
    }

    #[test]
    fn parsing_fqn_with_leading_colons() {
        let fqn = String::from("::full::contract::path::Contract");

        let path: syn::Path = syn::parse_quote! {
            ::full::contract::path::Contract
        };
        assert_eq!(path, fqn_to_path(&fqn));
    }
}
