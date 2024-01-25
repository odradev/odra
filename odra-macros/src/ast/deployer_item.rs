use derive_try_from_ref::TryFromRef;

use crate::{ir::ModuleImplIR, utils};

use super::deployer_utils::{
    CallEpcExpr, DeployerInitSignature, DeployerLoadSignature, EntrypointCallerExpr,
    EntrypointsInitExpr, EpcSignature, HostRefInstanceExpr, LoadContractExpr, NewContractExpr
};

#[derive(syn_derive::ToTokens)]
struct DeployStructItem {
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    semi_token: syn::token::Semi
}

impl TryFrom<&'_ ModuleImplIR> for DeployStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.deployer_ident()?,
            semi_token: Default::default()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct DeployImplItem {
    impl_token: syn::token::Impl,
    ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    epc_fn: ContractEpcFn,
    #[syn(in = brace_token)]
    init_fn: ContractInitFn,
    #[syn(in = brace_token)]
    load_fn: ContractLoadFn
}

impl TryFrom<&'_ ModuleImplIR> for DeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ident: module.deployer_ident()?,
            brace_token: Default::default(),
            epc_fn: module.try_into()?,
            init_fn: module.try_into()?,
            load_fn: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct ContractEpcFn {
    #[expr(utils::syn::visibility_pub())]
    vis: syn::Visibility,
    sig: EpcSignature,
    #[syn(braced)]
    #[default]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    entry_points: EntrypointsInitExpr,
    #[syn(in = braces)]
    caller: EntrypointCallerExpr
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
struct ContractInitFn {
    #[expr(utils::syn::visibility_pub())]
    vis: syn::Visibility,
    sig: DeployerInitSignature,
    #[syn(braced)]
    #[default]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    caller: CallEpcExpr,
    #[syn(in = braces)]
    new_contract: NewContractExpr,
    #[syn(in = braces)]
    host_ref_instance: HostRefInstanceExpr
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
struct ContractLoadFn {
    #[expr(utils::syn::visibility_pub())]
    vis: syn::Visibility,
    sig: DeployerLoadSignature,
    #[syn(braced)]
    #[default]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    caller: CallEpcExpr,
    #[syn(in = braces)]
    load_contract: LoadContractExpr,
    #[syn(in = braces)]
    host_ref_instance: HostRefInstanceExpr
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct DeployerItem {
    struct_item: DeployStructItem,
    impl_item: DeployImplItem
}

#[cfg(test)]
mod deployer_impl {
    use super::DeployerItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn deployer_impl() {
        let module = test_utils::mock::module_impl();
        let expected = quote! {
            pub struct Erc20Deployer;

            impl Erc20Deployer {
                pub fn epc(env: &odra::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(
                            odra::prelude::string::String::from("init"),
                            odra::prelude::vec![
                                odra::entry_point_callback::EntryPointArgument::new(
                                    odra::prelude::string::String::from("total_supply"),
                                    <Option::<U256> as odra::casper_types::CLTyped>::cl_type()
                                )
                            ]
                        ),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(
                            odra::prelude::string::String::from("approve"),
                            odra::prelude::vec![
                                odra::entry_point_callback::EntryPointArgument::new(
                                    odra::prelude::string::String::from("to"),
                                    <Address as odra::casper_types::CLTyped>::cl_type()
                                ),
                                odra::entry_point_callback::EntryPointArgument::new(
                                    odra::prelude::string::String::from("amount"),
                                    <U256 as odra::casper_types::CLTyped>::cl_type()
                                )
                            ]
                        ),
                        odra::entry_point_callback::EntryPoint::new(
                            odra::prelude::string::String::from("airdrop"),
                            odra::prelude::vec![
                                odra::entry_point_callback::EntryPointArgument::new(
                                    odra::prelude::string::String::from("to"),
                                    <odra::prelude::vec::Vec<Address> as odra::casper_types::CLTyped>::cl_type()
                                ),
                                odra::entry_point_callback::EntryPointArgument::new(
                                    odra::prelude::string::String::from("amount"),
                                    <U256 as odra::casper_types::CLTyped>::cl_type()
                                )
                            ])
                    ];

                    odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                        match call_def.entry_point() {
                            "init" => {
                                let result = __erc20_exec_parts::execute_init(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "pay_to_mint" => {
                                let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "approve" => {
                                let result = __erc20_exec_parts::execute_approve(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "airdrop" => {
                                let result = __erc20_exec_parts::execute_airdrop(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            name => Err(odra::OdraError::VmError(
                                odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                            ))
                        }
                    })
                }

                pub fn init(env: &odra::HostEnv, total_supply: Option<U256>) -> Erc20HostRef {
                    let caller = Self::epc(env);

                    let address = env.new_contract(
                        "Erc20",
                        Some({
                            let mut named_args = odra::casper_types::RuntimeArgs::new();
                            let _ = named_args.insert("total_supply", total_supply);
                            named_args
                        }),
                        caller
                    );
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                pub fn load(env: &odra::HostEnv, address: odra::Address) -> Erc20HostRef {
                    let caller = Self::epc(env);
                    env.register_contract(address, caller);
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::casper_types::U512::zero(),
                    }
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
            pub struct Erc20Deployer;

            impl Erc20Deployer {
                pub fn epc(env: &odra::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![])
                    ];
                    odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                        match call_def.entry_point() {
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "pay_to_mint" => {
                                let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            name => Err(odra::OdraError::VmError(
                                odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                            ))
                        }
                    })
                }

                pub fn init(env: &odra::HostEnv) -> Erc20HostRef {
                    let caller = Self::epc(env);

                    let address = env.new_contract(
                        "Erc20",
                        None,
                        caller
                    );
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                pub fn load(env: &odra::HostEnv, address: odra::Address) -> Erc20HostRef {
                    let caller = Self::epc(env);
                    env.register_contract(address, caller);
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::casper_types::U512::zero(),
                    }
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
            pub struct Erc20Deployer;

            impl Erc20Deployer {
                pub fn epc(env: &odra::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("get_owner"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("set_owner"), odra::prelude::vec![
                            odra::entry_point_callback::EntryPointArgument::new(odra::prelude::string::String::from("new_owner"), <Address as odra::casper_types::CLTyped>::cl_type())
                        ]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("name"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("symbol"), odra::prelude::vec![])
                    ];
                    odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                        match call_def.entry_point() {
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "get_owner" => {
                                let result = __erc20_exec_parts::execute_get_owner(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "set_owner" => {
                                let result = __erc20_exec_parts::execute_set_owner(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "name" => {
                                let result = __erc20_exec_parts::execute_name(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            "symbol" => {
                                let result = __erc20_exec_parts::execute_symbol(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                            }
                            name => Err(odra::OdraError::VmError(
                                odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                            ))
                        }
                    })
                }

                pub fn init(env: &odra::HostEnv) -> Erc20HostRef {
                    let caller = Self::epc(env);

                    let address = env.new_contract(
                        "Erc20",
                        None,
                        caller
                    );
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                pub fn load(env: &odra::HostEnv, address: odra::Address) -> Erc20HostRef {
                    let caller = Self::epc(env);
                    env.register_contract(address, caller);
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::casper_types::U512::zero(),
                    }
                }
            }
        };
        let deployer_item = DeployerItem::try_from(&module).unwrap();
        test_utils::assert_eq(deployer_item, &expected);
    }
}
