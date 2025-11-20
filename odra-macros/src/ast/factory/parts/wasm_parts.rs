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
        let entry_points = &self.entry_points;
        tokens.append_all(quote::quote! {
            #(#attrs)*
            mod #ident {
                #use_super
                #use_prelude

                #entry_points_fn

                #call_fn
                #factory_fn

                #(#entry_points)*
            }
        });
    }
}

struct FactoryEntrypointsFnItem {
    items: Vec<AddEntryPointStmtItem<FactoryContext>>,
    add_factory_entry_point: AddEntryPointStmtItem<FactoryContext>
}

impl ToTokens for FactoryEntrypointsFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_entry_points = utils::ty::entry_points();
        let ident_entry_points = utils::ident::entry_points();
        let expr_entry_points = utils::expr::new_entry_points();
        let items = &self.items;
        let add_factory_entry_point = &self.add_factory_entry_point;
        tokens.append_all(quote::quote! {
            #[inline]
            fn #ident_entry_points() -> #ty_entry_points {
                let mut #ident_entry_points = #expr_entry_points;
                #(#items)*
                #add_factory_entry_point
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
            add_factory_entry_point: AddEntryPointStmtItem::try_from(module)?
        })
    }
}

trait EntryPointContext {}

struct FactoryContext;
impl EntryPointContext for FactoryContext {}

struct InstallerContext;
impl EntryPointContext for InstallerContext {}

struct AddEntryPointStmtItem<Ctx: EntryPointContext> {
    entry_point_params: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
    ty: std::marker::PhantomData<Ctx>
}

impl<T: EntryPointContext> ToTokens for AddEntryPointStmtItem<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let var_ident = utils::ident::entry_points();
        let fn_ident = utils::ident::add_entry_point();
        let entry_point = utils::ty::entry_point();
        let params = &self.entry_point_params;
        tokens.append_all(quote::quote! {
            #var_ident.#fn_ident(#entry_point::new(
                #params
            ));
        });
    }
}

impl TryFrom<&'_ FnIR> for AddEntryPointStmtItem<FactoryContext> {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        let func_name = func.name_str();
        let param_name = parse_quote!(#func_name);
        let mut entry_point_params = syn::punctuated::Punctuated::new();
        entry_point_params.extend(vec![
            param_name,
            wasm_parts_utils::param_parameters(func),
            wasm_parts_utils::param_ret_ty(func),
            utils::expr::entry_point_access_template(),
            utils::expr::entry_point_contract(),
            utils::expr::entry_point_payment(),
        ]);
        Ok(Self {
            entry_point_params,
            ty: std::marker::PhantomData
        })
    }
}

impl TryFrom<&'_ FnIR> for AddEntryPointStmtItem<InstallerContext> {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        let func_name = func.name_str();
        let param_name = parse_quote!(#func_name);
        let mut entry_point_params = syn::punctuated::Punctuated::new();
        entry_point_params.extend(vec![
            param_name,
            wasm_parts_utils::param_parameters(func),
            wasm_parts_utils::param_ret_ty(func),
            wasm_parts_utils::param_access(func),
            utils::expr::entry_point_contract(),
            utils::expr::entry_point_payment(),
        ]);
        Ok(Self {
            entry_point_params,
            ty: std::marker::PhantomData
        })
    }
}

impl TryFrom<&'_ ModuleImplIR> for AddEntryPointStmtItem<FactoryContext> {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let func = module.constructor().unwrap();
        let param_name = parse_quote!("factory");
        let mut entry_point_params = syn::punctuated::Punctuated::new();
        entry_point_params.extend(vec![
            param_name,
            wasm_parts_utils::param_parameters(&func),
            utils::expr::key_cl_type(),
            utils::expr::entry_point_access_public(),
            utils::expr::entry_point_factory(),
            utils::expr::entry_point_payment(),
        ]);
        Ok(Self {
            entry_point_params,
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
    add_entry_point_items: Vec<AddEntryPointStmtItem<InstallerContext>>
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
            add_entry_point_items: module
                .functions()?
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?
        })
    }
}

impl ToTokens for NoMangleFactoryFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident_entry_points = utils::ident::entry_points();
        let ident_schemas = utils::ident::schemas();
        let address_ty = utils::ty::address();
        let string_ty = utils::ty::string();
        let ident = &self.module_ident.to_string();
        let contract_ident = ident.strip_suffix("Factory").unwrap_or(ident);
        let contract_ident: syn::Ident = format_ident!("{}", contract_ident);
        let events_expr = utils::expr::event_schemas(&contract_ident.as_type());
        let expr_new_schemas = utils::expr::schemas(&events_expr);
        let ident_args = utils::ident::named_args();
        let expr_entry_points = utils::expr::new_entry_points();
        let add_entry_point_items = &self.add_entry_point_items;
        let new_runtime_args = utils::expr::new_runtime_args();
        let insert_args = if let Some(fun) = self.init_fn.as_ref() {
            fn_utils::insert_args_stmts(fun, wasm_parts_utils::insert_arg_stmt)
        } else {
            vec![]
        };
        let event_ident = &self.event_ident;

        tokens.append_all(quote::quote! {
            #[no_mangle]
            fn factory() {
                let mut #ident_entry_points = #expr_entry_points;
                #(#add_entry_point_items)*
                
                let #ident_schemas = #expr_new_schemas;
                let exec_env = {
                    let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                    let env_rc = Rc::new(env);
                    odra::ExecutionEnv::new(env_rc)
                };
                let mut #ident_args = #new_runtime_args;
                #(#insert_args)*
        
                let (contract_package_hash, access_uref) = odra::odra_casper_wasm_env::host_functions::install_new_contract(
                    #ident_entry_points,
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
                    let mut entry_points = odra::casper_types::EntryPoints::new();
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "init",
                                vec![odra::args::parameter::<u32>("value")]
                                    .into_iter()
                                    .filter_map(|x| x)
                                    .collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Template,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                        .add_entry_point(odra::casper_types::EntityEntryPoint::new(
                            "total_supply",
                            vec![],
                            <U256 as odra::casper_types::CLTyped>::cl_type(),
                            odra::casper_types::EntryPointAccess::Template,
                            odra::casper_types::EntryPointType::Called,
                            odra::casper_types::EntryPointPayment::Caller,
                        )
                    );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "pay_to_mint",
                                vec![],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Template,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                         .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "approve",
                                vec![
                                    odra::args::parameter:: < Address > ("to"),
                                    odra::args::parameter:: < U256 > ("amount"),
                                    odra::args::parameter:: < Maybe < String > > ("msg")
                                ].into_iter().filter_map(|x| x).collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Template,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "factory",
                                vec![odra::args::parameter::<u32>("value")]
                                    .into_iter()
                                    .filter_map(|x| x)
                                    .collect(),
                                <odra::casper_types::Key as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Factory,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
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
                fn factory() {
                    let mut entry_points = odra::casper_types::EntryPoints::new();
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "init",
                                vec![odra::args::parameter::<u32>("value")]
                                    .into_iter()
                                    .filter_map(|x| x)
                                    .collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Groups(vec![odra::casper_types::Group::new("constructor_group")]),
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                        .add_entry_point(odra::casper_types::EntityEntryPoint::new(
                            "total_supply",
                            vec![],
                            <U256 as odra::casper_types::CLTyped>::cl_type(),
                            odra::casper_types::EntryPointAccess::Public,
                            odra::casper_types::EntryPointType::Called,
                            odra::casper_types::EntryPointPayment::Caller,
                        )
                    );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "pay_to_mint",
                                vec![],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
                    entry_points
                         .add_entry_point(
                            odra::casper_types::EntityEntryPoint::new(
                                "approve",
                                vec![
                                    odra::args::parameter:: < Address > ("to"),
                                    odra::args::parameter:: < U256 > ("amount"),
                                    odra::args::parameter:: < Maybe < String > > ("msg")
                                ].into_iter().filter_map(|x| x).collect(),
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Called,
                                odra::casper_types::EntryPointPayment::Caller,
                            ),
                        );
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
                        entry_points,
                        schemas,
                        Some(named_args)
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
