use crate::test_utils;
use quote::quote;

use super::HasEntrypointsImplItem;

#[test]
fn test_entrypoints() {
    let module = test_utils::mock::module_impl();
    let expected = quote!(
        impl odra::contract_def::HasEntrypoints for Erc20 {
            fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                odra::prelude::vec![
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("init"),
                        args: odra::prelude::vec![
                            odra::args::odra_argument::<Option<U256> >("total_supply")
                        ],
                        is_mutable: true,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Constructor,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("upgrade"),
                        args: odra::prelude::vec![
                            odra::args::odra_argument::<Option<U256> >("total_supply")
                        ],
                        is_mutable: true,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Constructor,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("total_supply"),
                        args: odra::prelude::vec![],
                        is_mutable: false,
                        return_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("pay_to_mint"),
                        args: odra::prelude::vec![],
                        is_mutable: true,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![odra::contract_def::EntrypointAttribute::Payable]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("approve"),
                        args: odra::prelude::vec![
                            odra::args::odra_argument::<Address>("to"),
                            odra::args::odra_argument::<U256>("amount"),
                            odra::args::odra_argument::<Maybe<String> >("msg")
                        ],
                        is_mutable: true,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![odra::contract_def::EntrypointAttribute::NonReentrant]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("airdrop"),
                        args: odra::prelude::vec![
                            odra::args::odra_argument::<odra::prelude::vec::Vec<Address> >("to"),
                            odra::args::odra_argument::<U256>("amount")
                        ],
                        is_mutable: false,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("swap"),
                        args: odra::prelude::vec![
                            odra::args::odra_argument::<Address>("to"),
                            odra::args::odra_argument::<U256>("amount")
                        ],
                        is_mutable: true,
                        return_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    }
                ]
            }
        }
    );
    let actual = HasEntrypointsImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_trait_impl_entrypoints() {
    let module = test_utils::mock::module_trait_impl();
    let expected = quote!(
        impl odra::contract_def::HasEntrypoints for Erc20 {
            fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                odra::prelude::vec![
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("total_supply"),
                        args: odra::prelude::vec![],
                        is_mutable: false,
                        return_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("set_total_supply"),
                        args: odra::prelude::vec![],
                        is_mutable: true,
                        return_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("pay_to_mint"),
                        args: odra::prelude::vec![],
                        is_mutable: true,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![odra::contract_def::EntrypointAttribute::Payable]
                    }
                ]
            }
        }
    );
    let actual = HasEntrypointsImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_delegated_entrypoints() {
    let module = test_utils::mock::module_delegation();
    let expected = quote!(
        impl odra::contract_def::HasEntrypoints for Erc20 {
            fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                odra::prelude::vec![
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("total_supply"),
                        args: odra::prelude::vec![],
                        is_mutable: false,
                        return_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("get_owner"),
                        args: odra::prelude::vec![],
                        is_mutable: false,
                        return_ty: <Address as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes : odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("set_owner"),
                        args: odra::prelude::vec![
                            odra::args::odra_argument::<Address>("new_owner")
                        ],
                        is_mutable: true,
                        return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("name"),
                        args: odra::prelude::vec![],
                        is_mutable: false,
                        return_ty: <String as odra::casper_types::CLTyped >::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes : odra::prelude::vec![]
                    },
                    odra::contract_def::Entrypoint {
                        name: odra::prelude::string::String::from("symbol"),
                        args: odra::prelude::vec![],
                        is_mutable: false,
                        return_ty: <String as odra::casper_types::CLTyped>::cl_type(),
                        ty: odra::contract_def::EntrypointType::Public,
                        attributes: odra::prelude::vec![]
                    }
                ]
            }
        }
    );
    let actual = HasEntrypointsImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
