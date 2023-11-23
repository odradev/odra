use crate::{ir::ModuleIR, utils};

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

impl TryFrom<&'_ ModuleIR> for DeployStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
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

impl TryFrom<&'_ ModuleIR> for DeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ident: module.deployer_ident()?,
            brace_token: Default::default(),
            init_fn: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ContractInitFn {
    vis: syn::Visibility,
    sig: DeployerInitSignature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    caller: EntrypointCallerExpr,
    #[syn(in = braces)]
    new_contract: NewContractExpr,
    #[syn(in = braces)]
    host_ref_instance: HostRefInstanceExpr
}

impl TryFrom<&'_ ModuleIR> for ContractInitFn {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            vis: utils::syn::visibility_pub(),
            sig: module.try_into()?,
            braces: Default::default(),
            caller: module.try_into()?,
            new_contract: module.try_into()?,
            host_ref_instance: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct DeployerItem {
    struct_item: DeployStructItem,
    impl_item: DeployImplItem
}

impl TryFrom<&'_ ModuleIR> for DeployerItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            struct_item: module.try_into()?,
            impl_item: module.try_into()?
        })
    }
}

#[cfg(test)]
mod deployer_impl {
    use super::DeployerItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn deployer_impl() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20Deployer;

            impl Erc20Deployer {
                pub fn init(env: &odra::HostEnv, total_supply: Option<U256>) -> Erc20HostRef {
                    let caller = odra::EntryPointsCaller::new(env.clone(), |contract_env, call_def| {
                        match call_def.method() {
                            "init" => {
                                let result = Erc20::new(Rc::new(contract_env))
                                    .init(call_def.get("total_supply").expect("arg not found"));
                                odra::ToBytes::to_bytes(&result).map(Into::into).unwrap()
                            }
                            "total_supply" => {
                                let result = Erc20::new(Rc::new(contract_env)).total_supply();
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
}
