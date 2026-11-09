use crate::test_utils;
use quote::quote;

use super::{FactorySchemaEntrypointsItem, SchemaEntrypointsItem};

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
                        false,
                        odra::prelude::vec![odra::schema::argument::<Option<U256> >(
                            "total_supply"
                        )]
                    ),
                    odra::schema::entry_point::<()>(
                        "upgrade",
                        "Upgrades the contract with the given parameters.",
                        true,
                        false,
                        odra::prelude::vec![odra::schema::argument::<Option<U256> >(
                            "total_supply"
                        )]
                    ),
                    odra::schema::entry_point::<U256>(
                        "total_supply",
                        "Returns the total supply of the token.",
                        false,
                        false,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<()>(
                        "pay_to_mint",
                        "Pay to mint.",
                        true,
                        true,
                        odra::prelude::vec![
                            odra::schema::argument::<odra::casper_types::URef>("__cargo_purse")
                        ]
                    ),
                    odra::schema::entry_point::<()>(
                        "approve",
                        "Approve.",
                        true,
                        false,
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
                        false,
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
fn test_entrypoint_with_multiline_docs() {
    let module = quote!(
        impl Erc20 {
            /// Initializes the contract.
            /// More details about init.
            pub fn init(&mut self) {}
        }
    );
    let attr = quote!();
    let module = crate::ir::ModuleImplIR::try_from((&attr, &module)).unwrap();
    let expected = quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaEntrypoints for Erc20 {
            fn schema_entrypoints(
            ) -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>
            {
                odra::prelude::vec![odra::schema::entry_point::<()>(
                    "init",
                    "Initializes the contract. More details about init.",
                    true,
                    false,
                    odra::prelude::vec![]
                )]
            }
        }
    );
    let actual = SchemaEntrypointsItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_entrypoint_with_paragraph_break_in_docs() {
    let module = quote!(
        impl Erc20 {
            /// Initializes the contract.
            ///
            /// Reverts if already initialized.
            pub fn init(&mut self) {}
        }
    );
    let attr = quote!();
    let module = crate::ir::ModuleImplIR::try_from((&attr, &module)).unwrap();
    let expected = quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaEntrypoints for Erc20 {
            fn schema_entrypoints(
            ) -> odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>
            {
                odra::prelude::vec![odra::schema::entry_point::<()>(
                    "init",
                    "Initializes the contract. Reverts if already initialized.",
                    true,
                    false,
                    odra::prelude::vec![]
                )]
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
                        false,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<U256>(
                        "set_total_supply",
                        "",
                        true,
                        false,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<()>(
                        "pay_to_mint",
                        "",
                        true,
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
                        false,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<Address>(
                        "get_owner",
                        "Returns the owner of the contract.",
                        false,
                        false,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<()>(
                        "set_owner",
                        "Sets the owner of the contract.",
                        true,
                        false,
                        odra::prelude::vec![odra::schema::argument::<Address>("new_owner")]
                    ),
                    odra::schema::entry_point::<String>(
                        "name",
                        "Returns the name of the token.",
                        false,
                        false,
                        odra::prelude::vec![]
                    ),
                    odra::schema::entry_point::<String>(
                        "symbol",
                        &odra::schema::delegated_description::<Self, _>(
                            |__module| &__module.metadata,
                            "symbol",
                            "Delegated. See `self.metadata.symbol()` for details."
                        ),
                        false,
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
                    odra::schema::entry_point::<()>("init", "", true, false, odra::prelude::vec![]),
                    odra::schema::entry_point::<(
                        odra::prelude::Address,
                        odra::casper_types::URef
                    )>(
                        "new_contract",
                        "",
                        true,
                        false,
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
                        false,
                        odra::prelude::vec![odra::schema::argument::<
                            odra::prelude::string::String
                        >("contract_name")]
                    ),
                    odra::schema::entry_point::<()>(
                        "batch_upgrade_child_contract",
                        "",
                        true,
                        false,
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
