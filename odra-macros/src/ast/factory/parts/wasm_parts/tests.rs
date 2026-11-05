use crate::ast::{wasm_parts::ModuleWasmPartsItem, Target};
use crate::test_utils;

#[test]
fn test() {
    let module = test_utils::mock::module_factory_impl();
    let actual = ModuleWasmPartsItem::try_new(&module, Target::Factory).expect("Failed to convert");

    let expected = quote::quote! {
        #[cfg(target_arch = "wasm32")]
        #[cfg(any(odra_module = "Erc20Factory", odra_module = "unknown_crate::Erc20Factory"))]
        mod __erc20_factory_wasm_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            fn entry_points() -> odra::casper_types::EntryPoints {
                use odra::entry_point::EntityEntryPointsExt;
                let mut entry_points = odra::casper_types::EntryPoints::new();
                entry_points.add(odra::entry_point::EntryPoint::Template {
                    name: "init",
                    args: vec![odra::args::parameter::<u32>("value")],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                });
                entry_points.add(odra::entry_point::EntryPoint::Template {
                    name: "total_supply",
                    args: vec![],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                });
                entry_points.add(odra::entry_point::EntryPoint::Template {
                    name: "pay_to_mint",
                    args: vec![],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                });
                entry_points.add(odra::entry_point::EntryPoint::Template {
                    name: "approve",
                    args: vec![
                        odra::args::parameter:: < Address > ("to"),
                        odra::args::parameter:: < U256 > ("amount"),
                        odra::args::parameter:: < Maybe < String > > ("msg")
                    ],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                });
                entry_points.add(odra::entry_point::EntryPoint::Factory {
                    args: vec![
                        odra::args::parameter::<odra::prelude::string::String>("contract_name"),
                        odra::args::parameter::<u32>("value")
                    ],
                });
                entry_points.add(odra::entry_point::EntryPoint::FactoryUpgrade {
                    args: vec![odra::args::parameter::<odra::prelude::string::String>("contract_name")],
                });
                entry_points.add(odra::entry_point::EntryPoint::FactoryBatchUpgrade);
                entry_points
            }

            #[inline]
            fn child_contract_entry_points() -> odra::casper_types::EntryPoints {
                use odra::entry_point::EntityEntryPointsExt;
                let mut entry_points = odra::casper_types::EntryPoints::new();
                entry_points.add(odra::entry_point::EntryPoint::Constructor {
                    args: vec![odra::args::parameter::<u32>("value")]
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "total_supply",
                    args: vec![],
                    ret_ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: false
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "pay_to_mint",
                    args: vec![],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: false,
                    is_payable: true
                });
                entry_points.add(odra::entry_point::EntryPoint::Regular {
                    name: "approve",
                    args: vec![
                        odra::args::parameter:: < Address > ("to"),
                        odra::args::parameter:: < U256 > ("amount"),
                        odra::args::parameter:: < Maybe < String > > ("msg")
                    ],
                    ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    is_non_reentrant: true,
                    is_payable: false
                });
                entry_points
            }

            #[no_mangle]
            fn call() {
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20Factory as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
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
                    Option::<odra::casper_types::RuntimeArgs>::None
                );
            }

            #[no_mangle]
            fn new_contract() {
                odra::odra_casper_wasm_env::host_functions::override_factory_caller();
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };

                let mut __odra_named_args = odra::casper_types::RuntimeArgs::new();
                odra::args::EntrypointArgument::insert_runtime_arg(
                    exec_env.get_named_arg::<u32>("value"),
                    "value",
                    &mut __odra_named_args
                );

                let (contract_package_hash, access_uref) = odra::odra_casper_wasm_env::host_functions::install_new_contract(
                    child_contract_entry_points(),
                    schemas,
                    None,
                    Some(__odra_named_args)
                );
                let address: odra::prelude::Address = contract_package_hash.into();

                exec_env.emit_event(Erc20FactoryContractDeployed {
                    contract_name: exec_env.get_named_arg::<odra::prelude::string::String>("contract_name"),
                    contract_address: address
                }); 
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t((address, access_uref))
                    )
                );
            }

            #[no_mangle]
            fn upgrade_child_contract() {
                use odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let name = exec_env.get_named_arg::<odra::prelude::string::String>("contract_name");
                let mut named_args = {
                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                    named_args
                };
                let contract_key = UnwrapOrRevert::unwrap_or_revert(
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(
                        &name,
                    ),
                );
                let package_hash = UnwrapOrRevert::unwrap_or_revert(
                    contract_key.into_package_hash(),
                );
                let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                    child_contract_entry_points(),
                    schemas,
                    None,
                    Some(named_args)
                );

                let address: odra::prelude::Address = contract_package_hash.into();
                exec_env.emit_event(Erc20FactoryContractDeployed {
                    contract_name: name,
                    contract_address: address
                });
            }

            #[no_mangle]
            fn batch_upgrade_child_contract() {
                use odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                use odra::casper_types::bytesrepr::FromBytes;
                let schemas = odra::casper_event_standard::Schemas(
                    <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                );
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let args = exec_env.get_named_arg::<odra::prelude::BTreeMap::<odra::prelude::string::String, odra::casper_types::bytesrepr::Bytes>>("args");

                for (name, args_bytes) in args {
                    let contract_key = odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(&name);
                    if let Some(key) = contract_key {
                        let package_hash = UnwrapOrRevert::unwrap_or_revert(key.into_package_hash());
                        let mut named_args: odra::casper_types::RuntimeArgs = UnwrapOrRevert::unwrap_or_revert(FromBytes::from_bytes(&args_bytes)).0;
                        let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                        let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                        let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                            child_contract_entry_points(),
                            schemas.clone(),
                            None,
                            Some(named_args)
                        );
                        let address: odra::prelude::Address = contract_package_hash.into();

                        exec_env.emit_event(Erc20FactoryContractDeployed {
                            contract_name: name,
                            contract_address: address
                        });
                    }
                }
            }

            #[no_mangle]
            fn init() {
                odra::odra_casper_wasm_env::host_functions::override_factory_caller();
                __erc20_factory_exec_parts::execute_init(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn total_supply() {
                let __odra_result = __erc20_factory_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t(__odra_result)
                    )
                );
            }

            #[no_mangle]
            fn pay_to_mint() {
                __erc20_factory_exec_parts::execute_pay_to_mint(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }

            #[no_mangle]
            fn approve() {
                __erc20_factory_exec_parts::execute_approve(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}
