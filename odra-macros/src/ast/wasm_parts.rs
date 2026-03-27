use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::utils::misc::AsType;
use crate::{
    ast::fn_utils,
    ir::{FnIR, ModuleImplIR},
    utils
};

use super::{
    parts_utils::{UsePreludeItem, UseSuperItem},
    wasm_parts_utils
};

#[derive(syn_derive::ToTokens)]
pub struct ModuleWasmPartsItem {
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    attrs: Vec<syn::Attribute>,
    mod_token: syn::token::Mod,
    ident: syn::Ident,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    use_super: UseSuperItem,
    #[syn(in = braces)]
    use_prelude: UsePreludeItem,
    #[syn(in = braces)]
    entry_points_fn: EntryPointsFnItem,
    #[syn(in = braces)]
    call_fn: CallFnItem,
    #[syn(in = braces)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    entry_points: Vec<NoMangleFnItem<ModuleContext>>
}

impl TryFrom<&'_ ModuleImplIR> for ModuleWasmPartsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_str = module.module_str()?;
        Ok(Self {
            attrs: vec![utils::attr::wasm32(), utils::attr::odra_module(&module_str)],
            mod_token: Default::default(),
            ident: module.wasm_parts_mod_ident()?,
            braces: Default::default(),
            use_super: UseSuperItem,
            use_prelude: UsePreludeItem,
            entry_points_fn: module.try_into()?,
            call_fn: module.try_into()?,
            entry_points: module
                .functions()?
                .iter()
                .map(|f| (module, f))
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct EntryPointsFnItem {
    inline_attr: syn::Attribute,
    sig: syn::Signature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    use_ext_import: syn::Stmt,
    #[syn(in = braces)]
    var_declaration: syn::Stmt,
    #[syn(in = braces)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    items: Vec<AddEntryPointStmtItem>,
    #[syn(in = braces)]
    ret: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for EntryPointsFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let ty_entry_points = utils::ty::entry_points();
        let ident_entry_points = utils::ident::entry_points();
        let expr_entry_points = utils::expr::new_entry_points();

        Ok(Self {
            inline_attr: utils::attr::inline(),
            sig: parse_quote!(fn #ident_entry_points() -> #ty_entry_points),
            braces: Default::default(),
            use_ext_import: wasm_parts_utils::use_entity_entry_points_ext(),
            var_declaration: parse_quote!(let mut #ident_entry_points = #expr_entry_points;),
            items: module
                .functions()?
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            ret: parse_quote!(#ident_entry_points)
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct CallFnItem {
    attr: syn::Attribute,
    sig: syn::Signature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    schemas_init_stmt: syn::Stmt,
    #[syn(in = braces)]
    exec_env_stmt: syn::Stmt,
    #[syn(in = braces)]
    is_upgrade_stmt: syn::Stmt,
    #[syn(in = braces)]
    runtime_args_stmt: syn::Stmt,
    #[syn(in = braces)]
    install_or_upgrade_stmt: syn::Stmt
}

impl TryFrom<&'_ ModuleImplIR> for CallFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_ident = module.module_ident()?.as_type();
        let ident_args = utils::ident::named_args();
        let ident_schemas = utils::ident::schemas();
        let ty_args = utils::ty::runtime_args();
        let ident_entry_points = utils::ident::entry_points();
        let exec_env_stmt: syn::Stmt = parse_quote!(
            let exec_env = {
                let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
                let env_rc = Rc::new(env);
                odra::ExecutionEnv::new(env_rc)
            };
        );
        let is_upgrade_stmt: syn::Stmt = parse_quote!(
            let is_upgrade = exec_env.get_named_arg::<bool>("odra_cfg_is_upgrade");
        );
        let runtime_args_expr: syn::Expr = match module.constructor() {
            Some(f) => {
                let arg_block = fn_utils::runtime_args_block(&f, wasm_parts_utils::insert_arg_stmt);
                parse_quote!({
                    Some(#arg_block)
                })
            }
            None => parse_quote!(Option::<#ty_args>::None)
        };

        let upgrade_args_expr: syn::Expr = match module.upgrader() {
            Some(f) => {
                let arg_block = fn_utils::runtime_args_block(&f, wasm_parts_utils::insert_arg_stmt);
                parse_quote!({
                    Some(#arg_block)
                })
            }
            None => parse_quote!(Option::<#ty_args>::None)
        };

        let args_stmt: syn::Stmt = parse_quote!(
            let #ident_args = if is_upgrade {
                #upgrade_args_expr
            } else {
                #runtime_args_expr
            };
        );
        let events_expr = utils::expr::event_schemas(&module_ident);
        let expr_new_schemas = utils::expr::schemas(&events_expr);
        let install_or_upgrade_stmt = utils::stmt::install_or_upgrade(
            parse_quote!(#ident_entry_points()),
            parse_quote!(#ident_schemas),
            parse_quote!(#ident_args)
        );

        Ok(Self {
            attr: utils::attr::no_mangle(),
            sig: parse_quote!(fn call()),
            braces: Default::default(),
            schemas_init_stmt: parse_quote!(let #ident_schemas = #expr_new_schemas;),
            exec_env_stmt,
            is_upgrade_stmt,
            runtime_args_stmt: args_stmt,
            install_or_upgrade_stmt
        })
    }
}

pub trait NoMangleItemContext {}

struct ModuleContext;
impl NoMangleItemContext for ModuleContext {}

pub(super) struct NoMangleFnItem<C: NoMangleItemContext> {
    pub sig: syn::Signature,
    pub override_stmt: Option<syn::Stmt>,
    pub execute_stmt: syn::Stmt,
    pub ret_stmt: Option<syn::Stmt>,
    pub ctx: std::marker::PhantomData<C>
}

impl<C: NoMangleItemContext> ToTokens for NoMangleFnItem<C> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let attr = utils::attr::no_mangle();
        let sig = &self.sig;
        let override_stmt = &self.override_stmt;
        let execute_stmt = &self.execute_stmt;
        let ret_stmt = &self.ret_stmt;
        tokens.append_all(quote::quote! {
            #attr
            #sig {
                #override_stmt
                #execute_stmt
                #ret_stmt
            }
        });
    }
}

impl TryFrom<(&'_ ModuleImplIR, &'_ FnIR)> for NoMangleFnItem<ModuleContext> {
    type Error = syn::Error;

    fn try_from(value: (&'_ ModuleImplIR, &'_ FnIR)) -> Result<Self, Self::Error> {
        let (module, func) = value;
        let fn_ident = func.name();
        let result_ident = utils::ident::result();
        let exec_parts_ident = module.exec_parts_mod_ident()?;
        let exec_fn = func.execute_name();
        let new_env = utils::expr::new_wasm_contract_env();

        let execute_stmt = match func.return_type() {
            syn::ReturnType::Default => parse_quote!(#exec_parts_ident::#exec_fn(#new_env);),
            syn::ReturnType::Type(_, _) => {
                parse_quote!(let #result_ident = #exec_parts_ident::#exec_fn(#new_env);)
            }
        };

        let ret_stmt = match func.return_type() {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, _) => Some(utils::stmt::runtime_return(&result_ident))
        };

        Ok(Self {
            sig: parse_quote!(fn #fn_ident()),
            override_stmt: None,
            execute_stmt,
            ret_stmt,
            ctx: std::marker::PhantomData::<ModuleContext>
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct AddEntryPointStmtItem {
    var_ident: syn::Ident,
    dot_token: syn::token::Dot,
    fn_ident: syn::Ident,
    #[syn(parenthesized)]
    paren: syn::token::Paren,
    #[syn(in = paren)]
    new_entry_point_expr: syn::Expr,
    semi_token: syn::token::Semi
}

impl TryFrom<&'_ FnIR> for AddEntryPointStmtItem {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        let args = wasm_parts_utils::param_parameters(func);
        let new_entry_point_expr = if func.is_constructor() {
            utils::expr::constructor_ep(args)
        } else if func.is_upgrader() {
            utils::expr::upgrader_ep(args)
        } else {
            utils::expr::regular_ep(
                func.name_str(),
                args,
                wasm_parts_utils::param_ret_ty(func),
                func.is_payable(),
                func.is_non_reentrant()
            )
        };
        Ok(Self {
            var_ident: utils::ident::entry_points(),
            dot_token: Default::default(),
            fn_ident: utils::ident::add_entry_point(),
            paren: Default::default(),
            new_entry_point_expr,
            semi_token: Default::default()
        })
    }
}

#[cfg(test)]
mod test {
    use super::ModuleWasmPartsItem;
    use crate::test_utils;

    #[test]
    fn test() {
        let module = test_utils::mock::module_impl();
        let actual = ModuleWasmPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
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

                    let named_args = if is_upgrade {
                        {
                            Some({
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                odra::args::EntrypointArgument::insert_runtime_arg(
                                    exec_env.get_named_arg::<Option<U256>>("total_supply"),
                                    "total_supply",
                                    &mut named_args,
                                );
                                named_args
                            })
                        }
                    } else {
                        {
                            Some({
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                odra::args::EntrypointArgument::insert_runtime_arg(
                                    exec_env.get_named_arg::<Option<U256>>("total_supply"),
                                    "total_supply",
                                    &mut named_args,
                                );
                                named_args
                            })
                        }
                    };

                    odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                        entry_points(),
                        schemas,
                        named_args
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
                    let result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
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
                    let result = __erc20_exec_parts::execute_swap(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
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
            #[cfg(odra_module = "Erc20")]
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
                    let named_args = if is_upgrade {
                        Option::<odra::casper_types::RuntimeArgs>::None
                    } else {
                        Option::<odra::casper_types::RuntimeArgs>::None
                    };
                    odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                        entry_points(),
                        schemas,
                        named_args
                    );
                }

                #[no_mangle]
                fn total_supply() {
                    let result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
                        )
                    );
                }

                #[no_mangle]
                fn set_total_supply() {
                    let result = __erc20_exec_parts::execute_set_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
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
            #[cfg(odra_module = "Erc20")]
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
                    let named_args = if is_upgrade {
                        Option::<odra::casper_types::RuntimeArgs>::None
                    } else {
                        Option::<odra::casper_types::RuntimeArgs>::None
                    };
                    odra::odra_casper_wasm_env::host_functions::install_or_upgrade(
                        entry_points(),
                        schemas,
                        named_args
                    );
                }

                #[no_mangle]
                fn total_supply() {
                    let result = __erc20_exec_parts::execute_total_supply(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result)
                        )
                    );
                }

                #[no_mangle]
                fn get_owner() {
                    let result = __erc20_exec_parts::execute_get_owner(
                        odra::odra_casper_wasm_env::WasmContractEnv::new_env(),
                    );
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result),
                        ),
                    );
                }

                #[no_mangle]
                fn set_owner() {
                    __erc20_exec_parts::execute_set_owner(odra::odra_casper_wasm_env::WasmContractEnv::new_env());
                }

                #[no_mangle]
                fn name() {
                    let result = __erc20_exec_parts::execute_name(
                        odra::odra_casper_wasm_env::WasmContractEnv::new_env(),
                    );
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result),
                        ),
                    );
                }

                #[no_mangle]
                fn symbol() {
                    let result = __erc20_exec_parts::execute_symbol(
                        odra::odra_casper_wasm_env::WasmContractEnv::new_env(),
                    );
                    odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
                        odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                            odra::casper_types::CLValue::from_t(result),
                        ),
                    );
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
