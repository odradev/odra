use derive_try_from::TryFromRef;

use crate::{ir::ModuleImplIR, utils};

use super::deployer_utils::{
    DeployerInitSignature, EntrypointCallerExpr, HostRefInstanceExpr, NewContractExpr
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
    init_fn: ContractInitFn
}

impl TryFrom<&'_ ModuleImplIR> for DeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ident: module.deployer_ident()?,
            brace_token: Default::default(),
            init_fn: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
struct ContractInitFn {
    #[expr(utils::syn::visibility_pub())]
    vis: syn::Visibility,
    sig: DeployerInitSignature,
    #[syn(braced)]
    #[default]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    caller: EntrypointCallerExpr,
    #[syn(in = braces)]
    new_contract: NewContractExpr,
    #[syn(in = braces)]
    host_ref_instance: HostRefInstanceExpr
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
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
                pub fn init(env: &odra::HostEnv, total_supply: Option<U256>) -> Erc20HostRef {
                    let caller = odra::EntryPointsCaller::new(env.clone(), |contract_env, call_def| {
                        match call_def.method() {
                            "init" => {
                                let result = __erc20_exec_parts::execute_init(contract_env);
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            "pay_to_mint" => {
                                let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            "approve" => {
                                let result = __erc20_exec_parts::execute_approve(contract_env);
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            _ => panic!("Unknown method")
                        }
                    });

                    let address = env.new_contract(
                        "Erc20",
                        Some({
                            let mut named_args = odra::RuntimeArgs::new();
                            let _ = named_args.insert("total_supply", total_supply);
                            named_args
                        }),
                        Some(caller)
                    );
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::U512::zero()
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
                pub fn init(env: &odra::HostEnv) -> Erc20HostRef {
                    let caller = odra::EntryPointsCaller::new(env.clone(), |contract_env, call_def| {
                        match call_def.method() {
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            "pay_to_mint" => {
                                let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            _ => panic!("Unknown method")
                        }
                    });

                    let address = env.new_contract(
                        "Erc20",
                        None,
                        Some(caller)
                    );
                    Erc20HostRef {
                        address,
                        env: env.clone(),
                        attached_value: odra::U512::zero()
                    }
                }
            }
        };
        let deployer_item = DeployerItem::try_from(&module).unwrap();
        test_utils::assert_eq(deployer_item, &expected);
    }
}
