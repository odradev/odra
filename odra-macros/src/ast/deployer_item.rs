#![allow(unused_variables)]

use crate::{ir::ModuleImplIR, utils};
use derive_try_from_ref::TryFromRef;
use quote::{ToTokens, TokenStreamExt};

use super::deployer_utils::{EntrypointCallerExpr, EntrypointsInitExpr, EpcSignature};

pub struct DeployImplItem {
    ident: syn::Ident,
    epc_fn: ContractEpcFn
}

impl TryFrom<&'_ ModuleImplIR> for DeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ident: module.host_ref_ident()?,
            epc_fn: module.try_into()?
        })
    }
}

impl ToTokens for DeployImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let epc_ty = utils::ty::entry_point_caller_provider();
        let ident = &self.ident;
        let epc_fn = &self.epc_fn;

        tokens.append_all(quote::quote! {
            impl #epc_ty for #ident {
                #epc_fn
            }
        });
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct ContractEpcFn {
    sig: EpcSignature,
    #[syn(braced)]
    #[default]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    entry_points: EntrypointsInitExpr,
    #[syn(in = braces)]
    caller: EntrypointCallerExpr
}

struct InitArgsItem {
    missing_docs: syn::Attribute,
    docs: syn::Attribute,
    attr: syn::Attribute,
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    braces: Option<syn::token::Brace>,
    fields: syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
    semi: Option<syn::token::Semi>,
    init_args_impl_item: InitArgsImplItem
}

impl quote::ToTokens for InitArgsItem {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        self.missing_docs.to_tokens(tokens);
        self.docs.to_tokens(tokens);
        self.attr.to_tokens(tokens);
        self.vis.to_tokens(tokens);
        self.struct_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        if let Some(ref braces) = self.braces {
            braces.surround(tokens, |tokens| {
                self.fields.to_tokens(tokens);
            });
        }
        self.semi.to_tokens(tokens);
        self.init_args_impl_item.to_tokens(tokens);
    }
}

impl TryFrom<&'_ ModuleImplIR> for InitArgsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let constructor = module.constructor().unwrap();
        let fields = constructor
            .named_args()
            .iter()
            .map(|arg| {
                let ty = utils::ty::unreferenced_ty(&arg.ty().unwrap());
                let ident = arg.name().unwrap();
                let field: syn::Field = syn::parse_quote!(pub #ident: #ty);
                field
            })
            .collect::<syn::punctuated::Punctuated<syn::Field, syn::Token![,]>>();
        let (braces, semi) = match fields.is_empty() {
            true => (None, Some(Default::default())),
            false => (Some(Default::default()), None)
        };
        Ok(Self {
            missing_docs: utils::attr::missing_docs(),
            docs: utils::attr::init_args_docs(module.module_str()?),
            attr: utils::attr::derive_into_runtime_args(),
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.init_args_ident()?,
            braces,
            fields,
            semi,
            init_args_impl_item: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct InitArgsImplItem {
    impl_token: syn::token::Impl,
    trait_ty: syn::Type,
    for_token: syn::token::For,
    ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace
}

impl TryFrom<&'_ ModuleImplIR> for InitArgsImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            trait_ty: utils::ty::init_args(),
            for_token: Default::default(),
            ident: module.init_args_ident()?,
            brace_token: Default::default()
        })
    }
}

struct UpgradeArgsItem {
    missing_docs: syn::Attribute,
    docs: syn::Attribute,
    attr: syn::Attribute,
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    braces: Option<syn::token::Brace>,
    fields: syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
    semi: Option<syn::token::Semi>,
    upgrade_args_impl_item: UpgradeArgsImplItem
}

impl quote::ToTokens for UpgradeArgsItem {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        self.missing_docs.to_tokens(tokens);
        self.docs.to_tokens(tokens);
        self.attr.to_tokens(tokens);
        self.vis.to_tokens(tokens);
        self.struct_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        if let Some(ref braces) = self.braces {
            braces.surround(tokens, |tokens| {
                self.fields.to_tokens(tokens);
            });
        }
        self.semi.to_tokens(tokens);
        self.upgrade_args_impl_item.to_tokens(tokens);
    }
}

impl TryFrom<&'_ ModuleImplIR> for UpgradeArgsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let upgrader = module.upgrader().unwrap();
        let fields = upgrader
            .named_args()
            .iter()
            .map(|arg| {
                let ty = utils::ty::unreferenced_ty(&arg.ty().unwrap());
                let ident = arg.name().unwrap();
                let field: syn::Field = syn::parse_quote!(pub #ident: #ty);
                field
            })
            .collect::<syn::punctuated::Punctuated<syn::Field, syn::Token![,]>>();
        let (braces, semi) = match fields.is_empty() {
            true => (None, Some(Default::default())),
            false => (Some(Default::default()), None)
        };
        Ok(Self {
            missing_docs: utils::attr::missing_docs(),
            docs: utils::attr::upgrade_args_docs(module.module_str()?),
            attr: utils::attr::derive_into_runtime_args(),
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.upgrade_args_ident()?,
            braces,
            fields,
            semi,
            upgrade_args_impl_item: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct UpgradeArgsImplItem {
    impl_token: syn::token::Impl,
    trait_ty: syn::Type,
    for_token: syn::token::For,
    ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace
}

impl TryFrom<&'_ ModuleImplIR> for UpgradeArgsImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            trait_ty: utils::ty::upgrade_args(),
            for_token: Default::default(),
            ident: module.upgrade_args_ident()?,
            brace_token: Default::default()
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct DeployerItem {
    args: Option<InitArgsItem>,
    upgrade_args: Option<UpgradeArgsItem>,
    impl_item: DeployImplItem
}

impl TryFrom<&'_ ModuleImplIR> for DeployerItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let args = match module.constructor() {
            Some(f) => {
                if f.has_args() {
                    Some(InitArgsItem::try_from(module)?)
                } else {
                    None
                }
            }
            None => None
        };
        let upgrade_args = match module.upgrader() {
            Some(f) => {
                if f.has_args() {
                    Some(UpgradeArgsItem::try_from(module)?)
                } else {
                    None
                }
            }
            None => None
        };
        Ok(Self {
            args,
            upgrade_args,
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
                fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
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

                    odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                        match call_def.entry_point() {
                            "init" => {
                                let result = __erc20_exec_parts::execute_init(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "upgrade" => {
                                let result = __erc20_exec_parts::execute_upgrade(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "pay_to_mint" => {
                                let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "approve" => {
                                let result = __erc20_exec_parts::execute_approve(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "airdrop" => {
                                let result = __erc20_exec_parts::execute_airdrop(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "swap" => {
                                let result = __erc20_exec_parts::execute_swap(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
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
                fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("set_total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new_payable(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![])
                    ];
                    odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                        match call_def.entry_point() {
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "set_total_supply" => {
                                let result = __erc20_exec_parts::execute_set_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "pay_to_mint" => {
                                let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
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
                fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("get_owner"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("set_owner"), odra::prelude::vec![
                            odra::entry_point_callback::Argument::new::<Address>(odra::prelude::string::String::from("new_owner"))
                        ]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("name"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("symbol"), odra::prelude::vec![])
                    ];
                    odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                        match call_def.entry_point() {
                            "total_supply" => {
                                let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "get_owner" => {
                                let result = __erc20_exec_parts::execute_get_owner(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "set_owner" => {
                                let result = __erc20_exec_parts::execute_set_owner(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "name" => {
                                let result = __erc20_exec_parts::execute_name(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                            }
                            "symbol" => {
                                let result = __erc20_exec_parts::execute_symbol(contract_env);
                                odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
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
}
