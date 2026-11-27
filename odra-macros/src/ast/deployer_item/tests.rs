use super::DeployerItem;
use crate::test_utils;
use quote::quote;

#[test]
fn deployer_impl() {
    let module = test_utils::mock::module_impl();
    let expected = quote! {
        #[allow(missing_docs)]
        /// [Erc20] contract constructor arguments.
        #[derive(odra::IntoRuntimeArgs)]
        pub struct Erc20InitArgs {
            pub total_supply: Option<U256>
        }

        impl odra::host::InitArgs for Erc20InitArgs {
        }

        #[allow(missing_docs)]
        /// [Erc20] contract upgrade arguments.
        #[derive(odra::IntoRuntimeArgs)]
        pub struct Erc20UpgradeArgs {
            pub total_supply: Option<U256>
        }
        impl odra::host::UpgradeArgs for Erc20UpgradeArgs {
        }

        impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
            fn entry_points_caller() -> odra::entry_point_callback::EntryPointsCaller {
                let entry_points = odra::prelude::vec![
                    odra::entry_point_callback::EntryPoint::new(
                        odra::prelude::string::String::from("init"),
                        odra::prelude::vec![
                            odra::entry_point_callback::Argument::new::<Option<U256> >(odra::prelude::string::String::from("total_supply"))
                        ]
                    ),
                    odra::entry_point_callback::EntryPoint::new(
                        odra::prelude::string::String::from("upgrade"),
                        odra::prelude::vec![
                            odra::entry_point_callback::Argument::new::<Option<U256> >(odra::prelude::string::String::from("total_supply"))
                        ]
                    ),
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new_payable(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new(
                        odra::prelude::string::String::from("approve"),
                        odra::prelude::vec![
                            odra::entry_point_callback::Argument::new::<Address>(odra::prelude::string::String::from("to")),
                            odra::entry_point_callback::Argument::new::<U256>(odra::prelude::string::String::from("amount")),
                            odra::entry_point_callback::Argument::new::<Maybe<String> >(odra::prelude::string::String::from("msg"))
                        ]
                    ),
                    odra::entry_point_callback::EntryPoint::new(
                        odra::prelude::string::String::from("airdrop"),
                        odra::prelude::vec![
                            odra::entry_point_callback::Argument::new::<odra::prelude::vec::Vec<Address> >(odra::prelude::string::String::from("to")),
                            odra::entry_point_callback::Argument::new::<U256>(odra::prelude::string::String::from("amount"))
                        ]
                    ),
                    odra::entry_point_callback::EntryPoint::new(
                        odra::prelude::string::String::from("swap"),
                        odra::prelude::vec![
                            odra::entry_point_callback::Argument::new::<Address>(odra::prelude::string::String::from("to")),
                            odra::entry_point_callback::Argument::new::<U256>(odra::prelude::string::String::from("amount"))
                        ]
                    )
                ];

                odra::entry_point_callback::EntryPointsCaller::new(entry_points, |__odra_contract_env, __odra_call_def| {
                    match __odra_call_def.entry_point() {
                        "init" => {
                            let __odra_result = __erc20_exec_parts::execute_init(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "upgrade" => {
                            let __odra_result = __erc20_exec_parts::execute_upgrade(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "total_supply" => {
                            let __odra_result = __erc20_exec_parts::execute_total_supply(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "pay_to_mint" => {
                            let __odra_result = __erc20_exec_parts::execute_pay_to_mint(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "approve" => {
                            let __odra_result = __erc20_exec_parts::execute_approve(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "airdrop" => {
                            let __odra_result = __erc20_exec_parts::execute_airdrop(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "swap" => {
                            let __odra_result = __erc20_exec_parts::execute_swap(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        name => Err(OdraError::VmError(
                            odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                        ))
                    }
                })
            }
        }
    };
    let deployer_item = DeployerItem::try_from(&module).unwrap();
    test_utils::assert_eq(deployer_item, &expected);
}

#[test]
fn deployer_trait_impl() {
    let module = test_utils::mock::module_trait_impl();
    let expected = quote! {
        impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
            fn entry_points_caller() -> odra::entry_point_callback::EntryPointsCaller {
                let entry_points = odra::prelude::vec![
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("set_total_supply"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new_payable(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![])
                ];
                odra::entry_point_callback::EntryPointsCaller::new(entry_points, |__odra_contract_env, __odra_call_def| {
                    match __odra_call_def.entry_point() {
                        "total_supply" => {
                            let __odra_result = __erc20_exec_parts::execute_total_supply(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "set_total_supply" => {
                            let __odra_result = __erc20_exec_parts::execute_set_total_supply(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "pay_to_mint" => {
                            let __odra_result = __erc20_exec_parts::execute_pay_to_mint(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        name => Err(OdraError::VmError(
                            odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                        ))
                    }
                })
            }
        }
    };
    let deployer_item = DeployerItem::try_from(&module).unwrap();
    test_utils::assert_eq(deployer_item, &expected);
}

#[test]
fn deployer_delegated() {
    let module = test_utils::mock::module_delegation();
    let expected = quote! {
        impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
            fn entry_points_caller() -> odra::entry_point_callback::EntryPointsCaller {
                let entry_points = odra::prelude::vec![
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("get_owner"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("set_owner"), odra::prelude::vec![
                        odra::entry_point_callback::Argument::new::<Address>(odra::prelude::string::String::from("new_owner"))
                    ]),
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("name"), odra::prelude::vec![]),
                    odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("symbol"), odra::prelude::vec![])
                ];
                odra::entry_point_callback::EntryPointsCaller::new(entry_points, |__odra_contract_env, __odra_call_def| {
                    match __odra_call_def.entry_point() {
                        "total_supply" => {
                            let __odra_result = __erc20_exec_parts::execute_total_supply(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "get_owner" => {
                            let __odra_result = __erc20_exec_parts::execute_get_owner(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "set_owner" => {
                            let __odra_result = __erc20_exec_parts::execute_set_owner(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "name" => {
                            let __odra_result = __erc20_exec_parts::execute_name(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        "symbol" => {
                            let __odra_result = __erc20_exec_parts::execute_symbol(__odra_contract_env);
                            odra::utils::try_serialize(&__odra_result)
                        }
                        name => Err(OdraError::VmError(
                            odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                        ))
                    }
                })
            }
        }
    };
    let deployer_item = DeployerItem::try_from(&module).unwrap();
    test_utils::assert_eq(deployer_item, &expected);
}

/// The host lists an `#[odra(offchain)]` function as an offchain entry point and dispatches it like the others.
#[test]
fn test_deployer_offchain_entry_point() {
    let module = test_utils::mock::module_offchain();
    let expected = quote! {
            impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                fn entry_points_caller() -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("balance_of"),
                        odra::prelude::vec![odra::entry_point_callback::Argument::new:: < Address
                        > (odra::prelude::string::String::from("owner"))]),
                        odra::entry_point_callback::EntryPoint::new_offchain(odra::prelude::string::String::from("balances"),
                        odra::prelude::vec![odra::entry_point_callback::Argument::new:: < Vec <
                        Address > > (odra::prelude::string::String::from("owners"))])
                    ];
                    odra::entry_point_callback::EntryPointsCaller::new(
                        entry_points,
                        |__odra_contract_env, __odra_call_def| {
                            match __odra_call_def.entry_point() {
                                "balance_of" => {
                                    let __odra_result = __erc20_exec_parts::execute_balance_of(
                                        __odra_contract_env,
                                    );
                                    odra::utils::try_serialize(&__odra_result)
                                }
                                "balances" => {
                                    let __odra_result = __erc20_exec_parts::execute_balances(
                                        __odra_contract_env,
                                    );
                                    odra::utils::try_serialize(&__odra_result)
                                }
                                name => {
                                    Err(
                                        OdraError::VmError(
                                            odra::VmError::NoSuchMethod(
                                                odra::prelude::String::from(name),
                                            ),
                                        ),
                                    )
                                }
                            }
                        },
                    )
                }
            }
    };
    let actual = DeployerItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

/// An args struct is emitted for `init` and for `upgrade` only when the function takes
/// arguments.
#[test]
fn deployer_args_structs_only_for_functions_with_args() {
    let deployer = |module| {
        let module = crate::ir::ModuleImplIR::try_from((&quote!(), &module)).unwrap();
        DeployerItem::try_from(&module).unwrap()
    };
    let count = |item: &DeployerItem, ident: &str| {
        let ident = quote::format_ident!("{ident}");
        test_utils::count_fragment(item, quote!(pub struct #ident))
    };

    let item = deployer(quote! {
        impl Erc20 {
            pub fn init(&mut self) {}
            pub fn upgrade(&mut self, admin: Address) {}
        }
    });
    assert_eq!(count(&item, "Erc20InitArgs"), 0);
    assert_eq!(count(&item, "Erc20UpgradeArgs"), 1);

    let item = deployer(quote! {
        impl Erc20 {
            pub fn init(&mut self, cap: u32) {}
            pub fn upgrade(&mut self) {}
        }
    });
    assert_eq!(count(&item, "Erc20InitArgs"), 1);
    assert_eq!(count(&item, "Erc20UpgradeArgs"), 0);
}
