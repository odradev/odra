use quote::ToTokens;

use crate::{
    ir::{FnIR, ModuleImplIR},
    utils::ty
};

pub struct SchemaEntrypointsItem {
    module_ident: syn::Ident,
    fns: Vec<FnIR>
}

impl ToTokens for SchemaEntrypointsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let fns = self
            .fns
            .iter()
            .map(|f| {
                let desc = f
                    .docs()
                    .first()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let name = f.name_str();
                let ret_ty = match f.return_type() {
                    syn::ReturnType::Default => quote::quote! { () },
                    syn::ReturnType::Type(_, t) => quote::quote! { #t }
                };
                let is_mut = f.is_mut();
                let args = args_to_tokens(&f.raw_typed_args());
                quote::quote! {
                    odra::schema::entry_point::<#ret_ty>(
                        #name,
                        #desc,
                        #is_mut,
                        odra::prelude::vec![ #(#args),* ]
                    )
                }
            })
            .collect::<Vec<_>>();

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for #module_ident {
                fn schema_entrypoints() -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint> {
                    odra::prelude::vec![ #(#fns),* ]
                }
            }
        };

        item.to_tokens(tokens);
    }
}

fn args_to_tokens(args: &[syn::PatType]) -> Vec<proc_macro2::TokenStream> {
    args.iter()
        .map(|syn::PatType { pat, ty, .. }| {
            let ty = ty::unreferenced_ty(ty);
            let name = pat.to_token_stream().to_string();
            quote::quote!(odra::schema::argument::<#ty>(#name))
        })
        .collect::<Vec<_>>()
}

impl TryFrom<&ModuleImplIR> for SchemaEntrypointsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident()?,
            fns: module.functions()?
        })
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::SchemaEntrypointsItem;

    #[test]
    fn test_entrypoints() {
        let module = test_utils::mock::module_impl();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for Erc20 {
                fn schema_entrypoints() -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint> {
                    odra::prelude::vec![
                        odra::schema::entry_point::<()>(
                            "init",
                            "Initializes the contract with the given parameters.",
                            true,
                            odra::prelude::vec![odra::schema::argument::<Option<U256> >("total_supply")]
                        ),
                        odra::schema::entry_point::<U256>(
                            "total_supply",
                            "Returns the total supply of the token.",
                            false,
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<()>(
                            "pay_to_mint",
                            "Pay to mint.",
                            true,
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<()>(
                            "approve",
                            "Approve.",
                            true,
                            odra::prelude::vec![
                                odra::schema::argument::<Address>("to"),
                                odra::schema::argument::<U256>("amount"),
                                odra::schema::argument::<Maybe<String> >("msg")
                            ]
                        ),
                        odra::schema::entry_point::<()>(
                            "airdrop",
                            "Airdrops the given amount to the given addresses.",
                            false,
                            odra::prelude::vec![
                                odra::schema::argument::<odra::prelude::vec::Vec<Address> >("to"),
                                odra::schema::argument::<U256>("amount")
                            ]
                        )
                    ]
                }
            }
        );
        let actual = SchemaEntrypointsItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_trait_impl_entrypoints() {
        let module = test_utils::mock::module_trait_impl();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for Erc20 {
                fn schema_entrypoints() -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint> {
                    odra::prelude::vec![
                        odra::schema::entry_point::<U256>("total_supply", "", false, odra::prelude::vec![]),
                        odra::schema::entry_point::<()>("pay_to_mint", "", true, odra::prelude::vec![])
                    ]
                }
            }
        );
        let actual = SchemaEntrypointsItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_delegated_entrypoints() {
        let module = test_utils::mock::module_delegation();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for Erc20 {
                fn schema_entrypoints() -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint> {
                    odra::prelude::vec![
                        odra::schema::entry_point::<U256>(
                            "total_supply",
                            "Returns the total supply of the token.",
                            false,
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<Address>(
                            "get_owner", 
                            "Returns the owner of the contract.", 
                            false, 
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<()>(
                            "set_owner",
                            "Sets the owner of the contract.",
                            true,
                            odra::prelude::vec![odra::schema::argument::<Address>("new_owner")]
                        ),
                        odra::schema::entry_point::<String>(
                            "name",
                            "Returns the name of the token.",
                            false, 
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<String>(
                            "symbol",
                            "Delegated. See `self.metadata.symbol()` for details.",
                            false,
                            odra::prelude::vec![]
                        )
                    ]
                }
            }
        );
        let actual = SchemaEntrypointsItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
