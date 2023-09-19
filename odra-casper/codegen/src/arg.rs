use odra_types::contract_def::Argument;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

pub(super) struct CasperArgs<'a>(pub &'a Vec<Argument>);

impl ToTokens for CasperArgs<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let args = self.0;
        args.iter().for_each(|arg| {
            let arg_ident = format_ident!("{}", arg.ident);
            let arg_name = &arg.ident;
            let ty = (arg.is_slice).then(|| quote!(: odra::prelude::vec::Vec<_>));
            tokens.append_all(quote! {
                let #arg_ident #ty = odra::casper::casper_contract::contract_api::runtime::get_named_arg(#arg_name);
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use odra_types::casper_types::CLType;

    use super::*;
    use crate::assert_eq_tokens;

    #[test]
    fn test_empty_args() {
        let args = vec![];
        let args = CasperArgs(&args);
        assert_eq_tokens(args, quote!());
    }

    #[test]
    fn test_two_args() {
        let args = vec![
            Argument {
                ident: String::from("a"),
                ty: CLType::Bool,
                is_ref: false,
                is_slice: false
            },
            Argument {
                ident: String::from("b_c"),
                ty: CLType::String,
                is_ref: false,
                is_slice: false
            },
        ];
        let args = CasperArgs(&args);
        assert_eq_tokens(
            args,
            quote!(
                let a = odra::casper::casper_contract::contract_api::runtime::get_named_arg("a");
                let b_c = odra::casper::casper_contract::contract_api::runtime::get_named_arg("b_c");
            )
        );
    }
}
