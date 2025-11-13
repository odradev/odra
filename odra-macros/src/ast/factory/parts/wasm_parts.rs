use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::ast::parts_utils::{UsePreludeItem, UseSuperItem};
use crate::ast::wasm_parts::NoMangleFnItem;
use crate::ast::wasm_parts_utils;
use crate::utils::misc::AsType;
use crate::{
    ast::fn_utils,
    ir::{FnIR, ModuleImplIR},
    utils
};

pub struct FactoryWasmPartsItem {
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    use_super: UseSuperItem,
    use_prelude: UsePreludeItem,
    entry_points_fn: FactoryEntrypointsFnItem,
    call_fn: CallFnItem,
    factory_fn: NoMangleFactoryFnItem,
    factory_upgrade_fn: NoMangleFactoryUpgradeFnItem,
    entry_points: Vec<NoMangleFnItem>
}

impl TryFrom<&'_ ModuleImplIR> for FactoryWasmPartsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_str = module.module_str()?;
        Ok(Self {
            attrs: vec![utils::attr::wasm32(), utils::attr::odra_module(&module_str)],
            ident: module.wasm_parts_mod_ident()?,
            use_super: UseSuperItem,
            use_prelude: UsePreludeItem,
            entry_points_fn: module.try_into()?,
            call_fn: module.try_into()?,
            factory_fn: module.try_into()?,
            factory_upgrade_fn: module.try_into()?,
            entry_points: module
                .functions()?
                .iter()
                .map(|f| (module, f))
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?
        })
    }
}

impl ToTokens for FactoryWasmPartsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let attrs = &self.attrs;
        let ident = &self.ident;
        let use_super = &self.use_super;
        let use_prelude = &self.use_prelude;
        let entry_points_fn = &self.entry_points_fn;
        let call_fn = &self.call_fn;
        let factory_fn = &self.factory_fn;
        let factory_upgrade_fn = &self.factory_upgrade_fn;
        let entry_points = &self.entry_points;
        tokens.append_all(quote::quote! {
            #(#attrs)*
            mod #ident {
                #use_super
                #use_prelude

                #entry_points_fn

                #call_fn
                #factory_fn
                #factory_upgrade_fn

                #(#entry_points)*
            }
        });
    }
}

struct FactoryEntrypointsFnItem {
    items: Vec<AddEntryPointStmtItem<FactoryContext>>,
    installer_items: Vec<AddEntryPointStmtItem<InstallerContext>>,
    add_factory_entry_point: AddEntryPointStmtItem<FactoryContext>,
    add_factory_upgrade_entry_point: AddEntryPointStmtItem<FactoryUpgradeContext>,
    add_factory_batch_upgrade_entry_point: AddEntryPointStmtItem<FactoryBatchUpgradeContext>
}

impl ToTokens for FactoryEntrypointsFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_entry_points = utils::ty::entry_points();
        let ident_entry_points = utils::ident::entry_points();
        let ident_child_contract_entry_points = utils::ident::child_contract_entry_points();
        let expr_entry_points = utils::expr::new_entry_points();
        let items = &self.items;
        let installer_items = &self.installer_items;
        let add_factory_entry_point = &self.add_factory_entry_point;
        let add_factory_upgrade_entry_point = &self.add_factory_upgrade_entry_point;
        let add_factory_batch_upgrade_entry_point = &self.add_factory_batch_upgrade_entry_point;
        let use_ext_import = wasm_parts_utils::use_entity_entry_points_ext();
        tokens.append_all(quote::quote! {
            #[inline]
            fn #ident_entry_points() -> #ty_entry_points {
                #use_ext_import
                let mut #ident_entry_points = #expr_entry_points;
                #(#items)*
                #add_factory_entry_point
                #add_factory_upgrade_entry_point
                #add_factory_batch_upgrade_entry_point
                #ident_entry_points
            }

            #[inline]
            fn #ident_child_contract_entry_points() -> #ty_entry_points {
                #use_ext_import
                let mut #ident_entry_points = #expr_entry_points;
                #(#installer_items)*
                #ident_entry_points
            }
        });
    }
}

impl TryFrom<&'_ ModuleImplIR> for FactoryEntrypointsFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            items: module
                .functions()?
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            installer_items: module
                .functions()?
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            add_factory_entry_point: AddEntryPointStmtItem::try_from(module)?,
            add_factory_upgrade_entry_point: AddEntryPointStmtItem::try_from(module)?,
            add_factory_batch_upgrade_entry_point: AddEntryPointStmtItem::try_from(module)?,
        })
    }
}

trait EntryPointContext {}

struct FactoryContext;
impl EntryPointContext for FactoryContext {}

struct FactoryUpgradeContext;
impl EntryPointContext for FactoryUpgradeContext {}

struct FactoryBatchUpgradeContext;
impl EntryPointContext for FactoryBatchUpgradeContext {}

struct InstallerContext;
impl EntryPointContext for InstallerContext {}

struct AddEntryPointStmtItem<Ctx: EntryPointContext> {
    entry_point_expr: syn::Expr,
    ty: std::marker::PhantomData<Ctx>
}

impl<T: EntryPointContext> ToTokens for AddEntryPointStmtItem<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let var_ident = utils::ident::entry_points();
        let fn_ident = utils::ident::add_entry_point();
        let entry_point_expr = &self.entry_point_expr;
        tokens.append_all(quote::quote! {
            #var_ident.#fn_ident(#entry_point_expr);
        });
    }
}

impl TryFrom<&'_ FnIR> for AddEntryPointStmtItem<FactoryContext> {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        let args = wasm_parts_utils::param_parameters(func);
     
        Ok(Self {
            entry_point_expr: utils::expr::template_ep(func.name_str(), args, wasm_parts_utils::param_ret_ty(func)),
            ty: std::marker::PhantomData
        })
    }
}

impl TryFrom<&'_ FnIR> for AddEntryPointStmtItem<InstallerContext> {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        let args = wasm_parts_utils::param_parameters(func);
        let entry_point_expr = if func.is_constructor() {
            utils::expr::constructor_ep(args)
        } else if func.is_upgrader() {
            utils::expr::upgrader_ep(args)
        } else {
            utils::expr::regular_ep(func.name_str(), args, wasm_parts_utils::param_ret_ty(func))
        };
        Ok(Self {
            entry_point_expr,
            ty: std::marker::PhantomData
        })
    }
}

impl TryFrom<&'_ ModuleImplIR> for AddEntryPointStmtItem<FactoryContext> {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let func = &module.factory_fn();
        Ok(Self {
            entry_point_expr: utils::expr::factory_ep(wasm_parts_utils::param_parameters(func)),
            ty: std::marker::PhantomData
        })
    }
}

impl TryFrom<&'_ ModuleImplIR> for AddEntryPointStmtItem<FactoryUpgradeContext> {
    type Error = syn::Error;

    fn try_from(_module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            entry_point_expr: utils::expr::factory_upgrade_ep(),
            ty: std::marker::PhantomData
        })
    }
}

impl TryFrom<&'_ ModuleImplIR> for AddEntryPointStmtItem<FactoryBatchUpgradeContext> {
    type Error = syn::Error;

    fn try_from(_module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            entry_point_expr: utils::expr::factory_batch_upgrade_ep(),
            ty: std::marker::PhantomData
        })
    }
}

struct CallFnItem {
    module_ident: syn::Ident
}

impl ToTokens for CallFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let attr = utils::attr::no_mangle();
        let ident_schemas = utils::ident::schemas();
        let events_expr = utils::expr::event_schemas(&self.module_ident.as_type());
        let expr_new_schemas = utils::expr::schemas(&events_expr);
        let ident_entry_points = utils::ident::entry_points();
        let ty_args = utils::ty::runtime_args();
        let install_or_upgrade_stmt = utils::stmt::install_or_upgrade(
            parse_quote!(#ident_entry_points()),
            parse_quote!(#ident_schemas),
            parse_quote!(Option::<#ty_args>::None)
        );

        tokens.append_all(quote::quote! {
            #attr
            fn call() {
                let #ident_schemas = #expr_new_schemas;
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                
                #install_or_upgrade_stmt
            }
        });
    }
}

impl TryFrom<&'_ ModuleImplIR> for CallFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident()?
        })
    }
}

struct NoMangleFactoryFnItem {
    module_ident: syn::Ident,
    event_ident: syn::Ident,
    init_fn: Option<FnIR>,
}

impl TryFrom<&'_ ModuleImplIR> for NoMangleFactoryFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_ident = module.module_ident()?;
        let module_str = module_ident.to_string();
        let event_ident = format_ident!("{}ContractDeployed", module_str);
        Ok(Self {
            module_ident,
            event_ident,
            init_fn: module.constructor(),
        })
    }
}

impl ToTokens for NoMangleFactoryFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident_entry_points = utils::ident::child_contract_entry_points();
        let ident_schemas = utils::ident::schemas();
        let address_ty = utils::ty::address();
        let string_ty = utils::ty::string();
        let ident = &self.module_ident.to_string();
        let contract_ident = ident.strip_suffix("Factory").unwrap_or(ident);
        let contract_ident: syn::Ident = format_ident!("{}", contract_ident);
        let events_expr = utils::expr::event_schemas(&contract_ident.as_type());
        let expr_new_schemas = utils::expr::schemas(&events_expr);
        let ident_args = utils::ident::named_args();
        let new_runtime_args = utils::expr::new_runtime_args();
        let insert_args = if let Some(fun) = self.init_fn.as_ref() {
            fn_utils::insert_args_stmts(fun, wasm_parts_utils::insert_arg_stmt)
        } else {
            vec![]
        };
        let event_ident = &self.event_ident;
        tokens.append_all(quote::quote! {
            #[no_mangle]
            fn new_contract() {
                let #ident_schemas = #expr_new_schemas;
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let mut #ident_args = #new_runtime_args;
                #(#insert_args)*
        
                let (contract_package_hash, access_uref) = odra::odra_casper_wasm_env::host_functions::install_new_contract(
                    #ident_entry_points(),
                    #ident_schemas,
                    Some(#ident_args)
                );
                let address: #address_ty = contract_package_hash.into();

                exec_env.emit_event(#event_ident {
                    contract_name: exec_env.get_named_arg::<#string_ty>("contract_name"),
                    contract_address: address
                });
                
                odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                    odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                        odra::casper_types::CLValue::from_t((address, access_uref))
                    )
                );
            }
        });
    }
}

struct NoMangleFactoryUpgradeFnItem {
    module_ident: syn::Ident,
    event_ident: syn::Ident,
    upgrader_args: Vec<(String, syn::Type)>,
}

impl TryFrom<&'_ ModuleImplIR> for NoMangleFactoryUpgradeFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_ident = module.module_ident()?;
        let module_str = module_ident.to_string();
        let upgrader_args = module.upgrader()
            .map(|fn_ir| fn_ir.named_args())
            .unwrap_or_default()
            .iter()
            .map(|arg| arg.name_and_ty())
            .collect::<Result<Vec<_>, _>>()?;
        let event_ident = format_ident!("{}ContractDeployed", module_str);
        Ok(Self {
            module_ident,
            event_ident,
            upgrader_args
        })
    }
}

impl ToTokens for NoMangleFactoryUpgradeFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident_entry_points = utils::ident::child_contract_entry_points();
        let ident_schemas = utils::ident::schemas();
        let address_ty = utils::ty::address();
        let string_ty = utils::ty::string();
        let vec_str_ty = utils::ty::vec_of(&string_ty);
        let runtime_args_ty = utils::ty::runtime_args();
        let btree_map_str_rt_ty = utils::ty::typed_btree_map(&string_ty, &runtime_args_ty);
        let bytes_ty = utils::ty::bytes();
        let ident = &self.module_ident.to_string();
        let contract_ident = ident.strip_suffix("Factory").unwrap_or(ident);
        let contract_ident: syn::Ident = format_ident!("{}", contract_ident);
        let events_expr = utils::expr::event_schemas(&contract_ident.as_type());
        let expr_new_schemas = utils::expr::schemas(&events_expr);
        let event_ident = &self.event_ident;
        let unwrap_or_revert = quote::quote!(odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert);
        let args = self.upgrader_args.iter().map(|(name, ty)| {
            quote::quote! { let _ = named_args.insert(#name, exec_env.get_named_arg::<#ty>(#name)); }
        }).collect::<TokenStream>();
        tokens.append_all(quote::quote! {
            #[no_mangle]
            fn upgrade_child_contract() {
                use #unwrap_or_revert;
                let #ident_schemas = #expr_new_schemas;
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let name = exec_env.get_named_arg::<#string_ty>("contract_name");
                let mut named_args = {
                    let mut named_args = #runtime_args_ty::new();
                    #args
                    named_args
                };
                let contract_key = UnwrapOrRevert::unwrap_or_revert(
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(&name)
                );
                let package_hash = UnwrapOrRevert::unwrap_or_revert(
                    contract_key.into_package_hash()
                );
                let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                    #ident_entry_points(),
                    #ident_schemas,
                    Some(named_args)
                );

                let address: Address = contract_package_hash.into();
                exec_env.emit_event(#event_ident {
                    contract_name: name,
                    contract_address: address
                });
            }

            #[no_mangle]
            fn batch_upgrade_child_contract() {
                use #unwrap_or_revert;
                use odra::casper_types::bytesrepr::FromBytes;

                let #ident_schemas = #expr_new_schemas;
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let default_args: #runtime_args_ty = UnwrapOrRevert::unwrap_or_revert(FromBytes::from_bytes(
                    &exec_env.get_named_arg::<#bytes_ty>("default_args")
                )).0;
                let specific_args: #btree_map_str_rt_ty = UnwrapOrRevert::unwrap_or_revert(FromBytes::from_bytes(
                    &exec_env.get_named_arg::<#bytes_ty>("specific_args")
                )).0;
                let names_to_upgrade: #vec_str_ty = UnwrapOrRevert::unwrap_or_revert(FromBytes::from_bytes(
                    &exec_env.get_named_arg::<#bytes_ty>("names_to_upgrade")
                )).0;

                for name in names_to_upgrade {
                    let contract_key = odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(&name);
                    if let Some(key) = contract_key {
                        let package_hash = UnwrapOrRevert::unwrap_or_revert(key.into_package_hash());
                        let mut named_args = specific_args.get(&name).cloned().unwrap_or_else(|| default_args.clone());
                        let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                        let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                        let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                            #ident_entry_points(),
                            #ident_schemas.clone(),
                            Some(named_args)
                        );
                        let address: #address_ty = contract_package_hash.into();

                        exec_env.emit_event(#event_ident {
                            contract_name: name,
                            contract_address: address
                        });
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::FactoryWasmPartsItem;
    use crate::test_utils;

    #[test]
    fn test() {
        let module = test_utils::mock::module_factory_impl();
        let actual = FactoryWasmPartsItem::try_from(&module).expect("Failed to convert");

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20Factory")]
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
                    entry_points.add(odra::entry_point::EntryPoint::FactoryUpgrade);
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
                    });
                    entry_points.add(odra::entry_point::EntryPoint::Regular {
                        name: "pay_to_mint",
                        args: vec![],
                        ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                    });
                    entry_points.add(odra::entry_point::EntryPoint::Regular {
                        name: "approve",
                        args: vec![
                            odra::args::parameter:: < Address > ("to"),
                            odra::args::parameter:: < U256 > ("amount"),
                            odra::args::parameter:: < Maybe < String > > ("msg")
                        ],
                        ret_ty: <() as odra::casper_types::CLTyped>::cl_type(),
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
                        Option::<odra::casper_types::RuntimeArgs>::None
                    );
                }

                #[no_mangle]
                fn new_contract() {
                    let schemas = odra::casper_event_standard::Schemas(
                        <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                    );
                    let exec_env = {
                        let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                        let env_rc = Rc::new(env);
                        odra::ExecutionEnv::new(env_rc)
                    };

                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                    odra::args::EntrypointArgument::insert_runtime_arg(
                        exec_env.get_named_arg::<u32>("value"),
                        "value",
                        &mut named_args
                    );

                    let (contract_package_hash, access_uref) = odra::odra_casper_wasm_env::host_functions::install_new_contract(
                        child_contract_entry_points(),
                        schemas,
                        Some(named_args)
                    );
                    let address: Address = contract_package_hash.into();

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
                    let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                        child_contract_entry_points(),
                        schemas,
                        Some(named_args)
                    );

                    let address: Address = contract_package_hash.into();
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

                    let default_args: odra::casper_types::RuntimeArgs = UnwrapOrRevert::unwrap_or_revert(
                            FromBytes::from_bytes(
                                &exec_env.get_named_arg::<odra::casper_types::bytesrepr::Bytes>("default_args"),
                            ),
                        )
                        .0;

                    let specific_args: odra::prelude::BTreeMap<
                        odra::prelude::string::String,
                        odra::casper_types::RuntimeArgs,
                    > = UnwrapOrRevert::unwrap_or_revert(
                            FromBytes::from_bytes(
                                &exec_env.get_named_arg::<odra::casper_types::bytesrepr::Bytes>("specific_args")
                            ),
                        )
                        .0;

                    let names_to_upgrade: odra::prelude::vec::Vec<
                        odra::prelude::string::String,
                    > = UnwrapOrRevert::unwrap_or_revert(
                            FromBytes::from_bytes(
                                &exec_env.get_named_arg::<odra::casper_types::bytesrepr::Bytes>("names_to_upgrade"),
                            ),
                        )
                        .0;

                    for name in names_to_upgrade {
                        let contract_key = odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_key(&name);
                        if let Some(key) = contract_key {
                            let package_hash = UnwrapOrRevert::unwrap_or_revert(key.into_package_hash());
                            let mut named_args = specific_args.get(&name).cloned().unwrap_or_else(|| default_args.clone());
                            let _ = named_args.insert("odra_cfg_package_hash_to_upgrade", package_hash.value());
                            let _ = named_args.insert("odra_cfg_package_hash_key_name", name.clone());
                            let contract_package_hash = odra::odra_casper_wasm_env::host_functions::upgrade_contract(
                                child_contract_entry_points(),
                                schemas.clone(),
                                Some(named_args)
                            );
                            let address: Address = contract_package_hash.into();

                            exec_env.emit_event(Erc20FactoryContractDeployed {
                                contract_name: name,
                                contract_address: address
                            });
                        }
                    }
                }

                #[no_mangle]
                fn init() {
                    __erc20_factory_exec_parts::execute_init(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                }

                #[no_mangle]
                fn total_supply() {
                    let result = __erc20_factory_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
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
}
