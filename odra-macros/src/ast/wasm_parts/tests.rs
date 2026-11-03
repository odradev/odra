use super::ModuleWasmPartsItem;
use crate::test_utils;

#[test]
fn test() {
    let module = test_utils::mock::module_impl();
    let actual = ModuleWasmPartsItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        #[cfg(target_arch = "wasm32")]
        #[cfg(any(odra_module = "Erc20", odra_module = "unknown_crate::Erc20"))]
        mod __erc20_wasm_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            fn entry_points() -> odra::casper_types::EntryPoints {
                use odra::entry_point::EntityEntryPointsExt;
                let mut entry_points = odra::casper_types::EntryPoints::new();
                entry_points.add(odra::entry_point::EntryPoint::Constructor {
                    args: vec![odra::args::parameter::<Option<U256> >("total_supply")],
                });
                entry_points.add(odra::entry_point::EntryPoint::Upgrader {
                    args: vec![odra::args::parameter::<Option<U256> >("total_supply")],
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "total_supply",
                    args: vec![],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "pay_to_mint",
                    args: vec![],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: true,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "approve",
                    args: vec![
                        odra::args::parameter::<Address>("to"),
                        odra::args::parameter::<U256>("amount"),
                        odra::args::parameter::<Maybe<String> >("msg")
                    ],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: true,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "airdrop",
                    args: vec![
                        odra::args::parameter::<odra::prelude::vec::Vec<Address> >("to"),
                        odra::args::parameter::<U256>("amount")
                    ],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "swap",
                    args: vec![
                        odra::args::parameter::<Address>("to"),
                        odra::args::parameter::<U256>("amount")
                    ],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points
            }

            #[no_mangle]
            fn call() {
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let is_upgrade = exec_env.get_named_arg::<bool>("odra_cfg_is_upgrade");

                let __odra_named_args = if is_upgrade {
                    {
                        Some({
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            odra::args::EntrypointArgument::insert_runtime_arg(
                                exec_env.get_named_arg::<Option<U256>>("total_supply"),
                                "total_supply",
                                &mut __odra_named_args,
                            );
                            __odra_named_args
                        })
                    }
                } else {
                    {
                        Some({
                            let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                            odra::args::EntrypointArgument::insert_runtime_arg(
                                exec_env.get_named_arg::<Option<U256>>("total_supply"),
                                "total_supply",
                                &mut __odra_named_args,
                            );
                            __odra_named_args
                        })
                    }
                };

                odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                    entry_points(),
                    schemas,
                    {
                        #[cfg(odra_contract_schema)]
                        let contract_schema = Some(include_str!(env!("ODRA_CONTRACT_SCHEMA_PATH")));
                        #[cfg(not(odra_contract_schema))]
                        let contract_schema: Option<&'static str> = None;
                        contract_schema
                    },
                    __odra_named_args
                );
            }

            #[no_mangle]
            fn init() {
                __erc20_exec_parts::execute_init(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn upgrade() {
                __erc20_exec_parts::execute_upgrade(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn total_supply() {
                let __odra_result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result)
                    )
                );
            }

            #[no_mangle]
            fn pay_to_mint() {
                __erc20_exec_parts::execute_pay_to_mint(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn approve() {
                __erc20_exec_parts::execute_approve(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn airdrop() {
                __erc20_exec_parts::execute_airdrop(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn swap() {
                let __odra_result = __erc20_exec_parts::execute_swap(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result)
                    )
                );
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_trait_impl() {
    let module = test_utils::mock::module_trait_impl();
    let actual = ModuleWasmPartsItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        #[cfg(target_arch = "wasm32")]
        #[cfg(any(odra_module = "Erc20", odra_module = "unknown_crate::Erc20"))]
        mod __erc20_wasm_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            fn entry_points() -> odra::casper_types::EntryPoints {
                use odra::entry_point::EntityEntryPointsExt;
                let mut entry_points = odra::casper_types::EntryPoints::new();
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "total_supply",
                    args: vec![],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "set_total_supply",
                    args: vec![],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "pay_to_mint",
                    args: vec![],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: true,
                });
                entry_points
            }

            #[no_mangle]
            fn call() {
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let is_upgrade = exec_env.get_named_arg::<bool>("odra_cfg_is_upgrade");
                let __odra_named_args = if is_upgrade {
                    Option::<odra::casper_types::RuntimeArgs>::None
                } else {
                    Option::<odra::casper_types::RuntimeArgs>::None
                };
                odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                    entry_points(),
                    schemas,
                    {
                        #[cfg(odra_contract_schema)]
                        let contract_schema = Some(include_str!(env!("ODRA_CONTRACT_SCHEMA_PATH")));
                        #[cfg(not(odra_contract_schema))]
                        let contract_schema: Option<&'static str> = None;
                        contract_schema
                    },
                    __odra_named_args
                );
            }

            #[no_mangle]
            fn total_supply() {
                let __odra_result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result)
                    )
                );
            }

            #[no_mangle]
            fn set_total_supply() {
                let __odra_result = __erc20_exec_parts::execute_set_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result)
                    )
                );
            }

            #[no_mangle]
            fn pay_to_mint() {
                __erc20_exec_parts::execute_pay_to_mint(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_delegate() {
    let module = test_utils::mock::module_delegation();
    let actual = ModuleWasmPartsItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        #[cfg(target_arch = "wasm32")]
        #[cfg(any(odra_module = "Erc20", odra_module = "unknown_crate::Erc20"))]
        mod __erc20_wasm_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            fn entry_points() -> odra::casper_types::EntryPoints {
                use odra::entry_point::EntityEntryPointsExt;
                let mut entry_points = odra::casper_types::EntryPoints::new();
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "total_supply",
                    args: vec![],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "get_owner",
                    args: vec![],
                    ret_ty: <Address as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "set_owner",
                    args: vec![odra::args::parameter::<Address>("new_owner")],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "name",
                    args: vec![],
                    ret_ty: <String as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "symbol",
                    args: vec![],
                    ret_ty: <String as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false,
                });
                entry_points
            }

            #[no_mangle]
            fn call() {
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let is_upgrade = exec_env.get_named_arg::<bool>("odra_cfg_is_upgrade");
                let __odra_named_args = if is_upgrade {
                    Option::<odra::casper_types::RuntimeArgs>::None
                } else {
                    Option::<odra::casper_types::RuntimeArgs>::None
                };
                odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                    entry_points(),
                    schemas,
                    {
                        #[cfg(odra_contract_schema)]
                        let contract_schema = Some(include_str!(env!("ODRA_CONTRACT_SCHEMA_PATH")));
                        #[cfg(not(odra_contract_schema))]
                        let contract_schema: Option<&'static str> = None;
                        contract_schema
                    },
                    __odra_named_args
                );
            }

            #[no_mangle]
            fn total_supply() {
                let __odra_result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result)
                    )
                );
            }

            #[no_mangle]
            fn get_owner() {
                let __odra_result = __erc20_exec_parts::execute_get_owner(
                    odra::odra_casper_wasm_env::WasmContractEnv::new_env(),
                );
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result),
                    ),
                );
            }

            #[no_mangle]
            fn set_owner() {
                __erc20_exec_parts::execute_set_owner(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn name() {
                let __odra_result = __erc20_exec_parts::execute_name(
                    odra::odra_casper_wasm_env::WasmContractEnv::new_env(),
                );
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result),
                    ),
                );
            }

            #[no_mangle]
            fn symbol() {
                let __odra_result = __erc20_exec_parts::execute_symbol(
                    odra::odra_casper_wasm_env::WasmContractEnv::new_env(),
                );
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result),
                    ),
                );
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}
