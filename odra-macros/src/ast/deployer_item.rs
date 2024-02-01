use crate::{ir::ModuleImplIR, utils};
use derive_try_from_ref::TryFromRef;
use quote::TokenStreamExt;

use super::deployer_utils::{EntrypointCallerExpr, EntrypointsInitExpr, EpcSignature};

#[derive(syn_derive::ToTokens)]
struct DeployImplItem {
    impl_token: syn::token::Impl,
    epc_provider_ty: syn::Type,
    for_token: syn::token::For,
    ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    epc_fn: ContractEpcFn
}

impl TryFrom<&'_ ModuleImplIR> for DeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            epc_provider_ty: utils::ty::entry_point_caller_provider(),
            for_token: Default::default(),
            ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            epc_fn: module.try_into()?
        })
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
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    braces: Option<syn::token::Brace>,
    fields: syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
    semi: Option<syn::token::Semi>
}

impl ::quote::ToTokens for InitArgsItem {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        tokens.append_all(self.attrs.iter());
        self.vis.to_tokens(tokens);
        self.struct_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        if let Some(ref braces) = self.braces {
            braces.surround(tokens, |tokens| {
                self.fields.to_tokens(tokens);
            });
        }
        self.semi.to_tokens(tokens);
    }
}

impl TryFrom<&'_ ModuleImplIR> for InitArgsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let constructor = module.constructor();

        let attrs = match constructor {
            Some(_) => vec![utils::attr::derive_into_runtime_args()],
            None => vec![
                utils::attr::derive_into_runtime_args(),
                utils::attr::is_none(),
            ]
        };
        let fields = match constructor {
            Some(constructor) => constructor
                .named_args()
                .iter()
                .map(|arg| {
                    let ty = arg.ty().unwrap();
                    let ident = arg.name().unwrap();
                    let field: syn::Field = syn::parse_quote!(pub #ident: #ty);
                    field
                })
                .collect::<syn::punctuated::Punctuated<syn::Field, syn::Token![,]>>(),
            None => Default::default()
        };
        let (braces, semi) = match fields.is_empty() {
            true => (None, Some(Default::default())),
            false => (Some(Default::default()), None)
        };
        Ok(Self {
            attrs,
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.init_args_ident()?,
            braces,
            fields,
            semi
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct DeployerItem {
    args: InitArgsItem,
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
            #[derive(odra::IntoRuntimeArgs)]
            pub struct Erc20InitArgs {
                pub total_supply: Option<U256>
            }

            impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(
                            odra::prelude::string::String::from("init"),
                            odra::prelude::vec![
                                odra::entry_point_callback::Argument::new(
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
                                odra::entry_point_callback::Argument::new(
                                    odra::prelude::string::String::from("to"),
                                    <Address as odra::casper_types::CLTyped>::cl_type()
                                ),
                                odra::entry_point_callback::Argument::new(
                                    odra::prelude::string::String::from("amount"),
                                    <U256 as odra::casper_types::CLTyped>::cl_type()
                                )
                            ]
                        ),
                        odra::entry_point_callback::EntryPoint::new(
                            odra::prelude::string::String::from("airdrop"),
                            odra::prelude::vec![
                                odra::entry_point_callback::Argument::new(
                                    odra::prelude::string::String::from("to"),
                                    <odra::prelude::vec::Vec<Address> as odra::casper_types::CLTyped>::cl_type()
                                ),
                                odra::entry_point_callback::Argument::new(
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
            }
        };
        let deployer_item = DeployerItem::try_from(&module).unwrap();
        test_utils::assert_eq(deployer_item, &expected);
    }

    #[test]
    fn deployer_trait_impl() {
        let module = test_utils::mock::module_trait_impl();
        let expected = quote! {
            #[derive(odra::IntoRuntimeArgs)]
            #[is_none]
            pub struct Erc20InitArgs;

            impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
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
            }
        };
        let deployer_item = DeployerItem::try_from(&module).unwrap();
        test_utils::assert_eq(deployer_item, &expected);
    }

    #[test]
    fn deployer_delegated() {
        let module = test_utils::mock::module_delegation();
        let expected = quote! {
            #[derive(odra::IntoRuntimeArgs)]
            #[is_none]
            pub struct Erc20InitArgs;

            impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("get_owner"), odra::prelude::vec![]),
                        odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("set_owner"), odra::prelude::vec![
                            odra::entry_point_callback::Argument::new(odra::prelude::string::String::from("new_owner"), <Address as odra::casper_types::CLTyped>::cl_type())
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
            }
        };
        let deployer_item = DeployerItem::try_from(&module).unwrap();
        test_utils::assert_eq(deployer_item, &expected);
    }
}
