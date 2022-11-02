use odra_types::contract_def::Argument;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

pub(super) struct CasperArgs<'a>(pub &'a Vec<Argument>);

impl ToTokens for CasperArgs<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let args = self.0;

        args.iter().for_each(|arg| {
            let arg_ident = format_ident!("{}", arg.ident);

            tokens.append_all(quote! {
                let #arg_ident = odra::casper::casper_contract::contract_api::runtime::get_named_arg(stringify!(#arg_ident));
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use odra_types::Type;

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
                ty: Type::Bool,
            },
            Argument {
                ident: String::from("b_c"),
                ty: Type::String,
            },
        ];
        let args = CasperArgs(&args);
        assert_eq_tokens(
            args,
            quote!(
                let a = odra::casper::casper_contract::contract_api::runtime::get_named_arg(stringify!(a));
                let b_c = odra::casper::casper_contract::contract_api::runtime::get_named_arg(stringify!(b_c));
            ),
        );
    }
}
