use quote::ToTokens;
use syn::{parse_quote, punctuated::Punctuated, Token};

use crate::{
    ir::{FnIR, FnTraitIR, FnType, ModuleImplIR},
    utils::{self, ty}
};

#[derive(syn_derive::ToTokens)]
pub struct FactorySchemaEntrypointsItem {
    item: SchemaEntrypointsItem
}

impl TryFrom<&ModuleImplIR> for FactorySchemaEntrypointsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let item = SchemaEntrypointsItem {
            module_ident: module.module_ident()?,
            fns: module
                .functions()?
                .into_iter()
                .filter(|f| f.is_constructor())
                .map(|f| {
                    let receiver: syn::FnArg = parse_quote!(&mut self);
                    let mut inputs = Punctuated::<syn::FnArg, Token![,]>::new();
                    inputs.push(receiver);
                    let argless_sig = match f {
                        FnIR::Impl(fn_impl_ir) => {
                            let sig = fn_impl_ir.sig();
                            syn::Signature {
                                inputs,
                                ..sig.clone()
                            }
                        }
                        FnIR::Def(fn_trait_ir) => {
                            let sig = fn_trait_ir.sig();
                            syn::Signature {
                                inputs,
                                ..sig.clone()
                            }
                        }
                    };
                    FnIR::Def(FnTraitIR::new(parse_quote!(#argless_sig;)))
                })
                .chain(vec![
                    module.factory_fn(),
                    module.factory_upgrade_fn(),
                    module.factory_batch_upgrade_fn(),
                ])
                .collect()
        };
        Ok(Self { item })
    }
}

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
                let mut args = if f.fn_type() == FnType::FactoryBatchUpgrader {
                    let ty_bytes = utils::ty::bytes();
                    vec![quote::quote!(odra::schema::argument::<#ty_bytes>("args"))]
                } else {
                    args_to_tokens(&f.raw_typed_args())
                };
                if f.is_payable() {
                    args.push(quote::quote!(odra::schema::argument::<
                        odra::casper_types::URef
                    >("__cargo_purse")))
                };
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
    use crate::{ast::schema::entry_points::FactorySchemaEntrypointsItem, test_utils};
    use quote::quote;

    use super::SchemaEntrypointsItem;

    #[test]
    fn test_entrypoints() {
        let module = test_utils::mock::module_impl();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for Erc20 {
                fn schema_entrypoints(
                ) -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>
                {
                    odra::prelude::vec![
                        odra::schema::entry_point::<()>(
                            "init",
                            "Initializes the contract with the given parameters.",
                            true,
                            odra::prelude::vec![odra::schema::argument::<Option<U256> >(
                                "total_supply"
                            )]
                        ),
                        odra::schema::entry_point::<()>(
                            "upgrade",
                            "Upgrades the contract with the given parameters.",
                            true,
                            odra::prelude::vec![odra::schema::argument::<Option<U256> >(
                                "total_supply"
                            )]
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
                            odra::prelude::vec![
                                odra::schema::argument::<odra::casper_types::URef>("__cargo_purse")
                            ]
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
                        ),
                        odra::schema::entry_point::<U256>(
                            "swap",
                            "Swaps the given amount to the given addresses.",
                            true,
                            odra::prelude::vec![
                                odra::schema::argument::<Address>("to"),
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
                fn schema_entrypoints(
                ) -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>
                {
                    odra::prelude::vec![
                        odra::schema::entry_point::<U256>(
                            "total_supply",
                            "",
                            false,
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<U256>(
                            "set_total_supply",
                            "",
                            true,
                            odra::prelude::vec![]
                        ),
                        odra::schema::entry_point::<()>(
                            "pay_to_mint",
                            "",
                            true,
                            odra::prelude::vec![
                                odra::schema::argument::<odra::casper_types::URef>("__cargo_purse")
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
    fn test_delegated_entrypoints() {
        let module = test_utils::mock::module_delegation();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for Erc20 {
                fn schema_entrypoints(
                ) -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>
                {
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

    #[test]
    fn test_factory_entrypoints() {
        let module = test_utils::mock::module_factory_impl();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for Erc20Factory {
                fn schema_entrypoints(
                ) -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>
                {
                    odra::prelude::vec![
                        odra::schema::entry_point::<()>("init", "", true, odra::prelude::vec![]),
                        odra::schema::entry_point::<(
                            odra::prelude::Address,
                            odra::casper_types::URef
                        )>(
                            "new_contract",
                            "",
                            true,
                            odra::prelude::vec![
                                odra::schema::argument::<odra::prelude::string::String>(
                                    "contract_name"
                                ),
                                odra::schema::argument::<u32>("value")
                            ]
                        ),
                        odra::schema::entry_point::<()>(
                            "upgrade_child_contract",
                            "",
                            true,
                            odra::prelude::vec![odra::schema::argument::<
                                odra::prelude::string::String
                            >("contract_name")]
                        ),
                        odra::schema::entry_point::<()>(
                            "batch_upgrade_child_contract",
                            "",
                            true,
                            odra::prelude::vec![odra::schema::argument::<
                                odra::casper_types::bytesrepr::Bytes
                            >("args")]
                        )
                    ]
                }
            }
        );
        let actual = FactorySchemaEntrypointsItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
