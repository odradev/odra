use quote::TokenStreamExt;
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
pub struct WasmPartsModuleItem {
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
    entry_points: Vec<NoMangleFnItem>
}

impl TryFrom<&'_ ModuleImplIR> for WasmPartsModuleItem {
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
    runtime_args_stmt: syn::Stmt,
    #[syn(in = braces)]
    install_contract_stmt: syn::Stmt
}

impl TryFrom<&'_ ModuleImplIR> for CallFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_ident = module.module_ident()?.as_type();
        let ident_args = utils::ident::named_args();
        let ident_schemas = utils::ident::schemas();
        let ty_args = utils::ty::runtime_args();
        let ident_entry_points = utils::ident::entry_points();
        let runtime_args_expr: syn::Expr = match module.constructor() {
            Some(f) => {
                let arg_block = fn_utils::runtime_args_block(&f, wasm_parts_utils::insert_arg_stmt);
                parse_quote!(let #ident_args = Some(#arg_block))
            }
            None => parse_quote!(let #ident_args = Option::<#ty_args>::None)
        };
        let events_expr = utils::expr::event_schemas(&module_ident);
        let expr_new_schemas = utils::expr::schemas(&events_expr);
        let install_contract_stmt = utils::stmt::install_contract(
            parse_quote!(#ident_entry_points()),
            parse_quote!(#ident_schemas),
            parse_quote!(#ident_args)
        );

        Ok(Self {
            attr: utils::attr::no_mangle(),
            sig: parse_quote!(fn call()),
            braces: Default::default(),
            schemas_init_stmt: parse_quote!(let #ident_schemas = #expr_new_schemas;),
            runtime_args_stmt: parse_quote!(#runtime_args_expr;),
            install_contract_stmt
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct NoMangleFnItem {
    attr: syn::Attribute,
    sig: syn::Signature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    execute_stmt: syn::Stmt,
    #[syn(in = braces)]
    ret_stmt: Option<syn::Stmt>
}

impl TryFrom<(&'_ ModuleImplIR, &'_ FnIR)> for NoMangleFnItem {
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
            attr: utils::attr::no_mangle(),
            sig: parse_quote!(fn #fn_ident()),
            braces: Default::default(),
            execute_stmt,
            ret_stmt
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
    new_entry_point_expr: NewEntryPointItem,
    semi_token: syn::token::Semi
}

impl TryFrom<&'_ FnIR> for AddEntryPointStmtItem {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        Ok(Self {
            var_ident: utils::ident::entry_points(),
            dot_token: Default::default(),
            fn_ident: utils::ident::add_entry_point(),
            paren: Default::default(),
            new_entry_point_expr: func.try_into()?,
            semi_token: Default::default()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct NewEntryPointItem {
    ty: syn::Type,
    colon_colon_token: syn::token::PathSep,
    new_ident: syn::Ident,
    #[syn(parenthesized)]
    paren: syn::token::Paren,
    #[syn(in = paren)]
    params: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>
}

impl TryFrom<&'_ FnIR> for NewEntryPointItem {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        let func_name = func.name_str();
        let param_name = parse_quote!(#func_name);
        let param_parameters = wasm_parts_utils::param_parameters(func);
        let param_ret_ty = wasm_parts_utils::param_ret_ty(func);
        let param_access = wasm_parts_utils::param_access(func);

        let mut params = syn::punctuated::Punctuated::new();
        params.extend(vec![
            param_name,
            param_parameters,
            param_ret_ty,
            param_access,
            utils::expr::entry_point_contract(),
        ]);
        Ok(Self {
            ty: utils::ty::entry_point(),
            colon_colon_token: Default::default(),
            new_ident: utils::ident::new(),
            paren: Default::default(),
            params
        })
    }
}

#[cfg(test)]
mod test {
    use super::WasmPartsModuleItem;
    use crate::test_utils;

    #[test]
    fn test() {
        let module = test_utils::mock::module_impl();
        let actual = WasmPartsModuleItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
            mod __erc20_wasm_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                fn entry_points() -> odra::casper_types::EntryPoints {
                    let mut entry_points = odra::casper_types::EntryPoints::new();

                    entry_points.add_entry_point(odra::casper_types::EntryPoint::new(
                        "init",
                        vec![odra::casper_types::Parameter::new(
                            "total_supply",
                            <Option::<U256> as odra::casper_types::CLTyped>::cl_type()
                        )],
                        <() as odra::casper_types::CLTyped>::cl_type(),
                        odra::casper_types::EntryPointAccess::Groups(vec![odra::casper_types::Group::new("constructor_group")]),
                        odra::casper_types::EntryPointType::Contract
                    ));
                    entry_points.add_entry_point(odra::casper_types::EntryPoint::new(
                        "total_supply",
                        vec![],
                        <U256 as odra::casper_types::CLTyped>::cl_type(),
                        odra::casper_types::EntryPointAccess::Public,
                        odra::casper_types::EntryPointType::Contract
                    ));
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "pay_to_mint",
                                vec![],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "approve",
                                vec![
                                    odra::casper_types::Parameter::new("to", < Address as
                                    odra::casper_types::CLTyped > ::cl_type()),
                                    odra::casper_types::Parameter::new("amount", < U256 as
                                    odra::casper_types::CLTyped > ::cl_type())
                                ],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "airdrop",
                                vec![
                                    odra::casper_types::Parameter::new("to", <odra::prelude::vec::Vec<Address> as
                                    odra::casper_types::CLTyped > ::cl_type()),
                                    odra::casper_types::Parameter::new("amount", <U256 as
                                    odra::casper_types::CLTyped > ::cl_type())
                                ],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                }

                #[no_mangle]
                fn call() {
                    let schemas = odra::casper_event_standard::Schemas(
                        <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                    );
                    let named_args = Some({
                        let mut named_args = odra::RuntimeArgs::new();
                        let _ = named_args.insert(
                            "total_supply",
                            odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_named_arg::<Option<U256>>("total_supply")
                        );
                        named_args
                    });
                    odra::odra_casper_wasm_env::host_functions::install_contract(
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
            }
        };

        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_trait_impl() {
        let module = test_utils::mock::module_trait_impl();
        let actual = WasmPartsModuleItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
            mod __erc20_wasm_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                fn entry_points() -> odra::casper_types::EntryPoints {
                    let mut entry_points = odra::casper_types::EntryPoints::new();
                    entry_points.add_entry_point(odra::casper_types::EntryPoint::new(
                        "total_supply",
                        vec![],
                        <U256 as odra::casper_types::CLTyped>::cl_type(),
                        odra::casper_types::EntryPointAccess::Public,
                        odra::casper_types::EntryPointType::Contract
                    ));
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "pay_to_mint",
                                vec![],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                }

                #[no_mangle]
                fn call() {
                    let schemas = odra::casper_event_standard::Schemas(
                        <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                    );
                    let named_args = Option::<odra::RuntimeArgs>::None;
                    odra::odra_casper_wasm_env::host_functions::install_contract(
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
        let actual = WasmPartsModuleItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
            mod __erc20_wasm_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                fn entry_points() -> odra::casper_types::EntryPoints {
                    let mut entry_points = odra::casper_types::EntryPoints::new();
                    entry_points.add_entry_point(odra::casper_types::EntryPoint::new(
                        "total_supply",
                        vec![],
                        <U256 as odra::casper_types::CLTyped>::cl_type(),
                        odra::casper_types::EntryPointAccess::Public,
                        odra::casper_types::EntryPointType::Contract
                    ));
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "get_owner",
                                vec![],
                                <Address as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                         .add_entry_point(
                             odra::casper_types::EntryPoint::new(
                                "set_owner",
                                vec![
                                    odra::casper_types::Parameter::new("new_owner", < Address as
                                    odra::casper_types::CLTyped > ::cl_type())
                                ],
                                <() as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "name",
                                vec![],
                                <String as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                        .add_entry_point(
                            odra::casper_types::EntryPoint::new(
                                "symbol",
                                vec![],
                                <String as odra::casper_types::CLTyped>::cl_type(),
                                odra::casper_types::EntryPointAccess::Public,
                                odra::casper_types::EntryPointType::Contract,
                            ),
                        );
                    entry_points
                }

                #[no_mangle]
                fn call() {
                    let schemas = odra::casper_event_standard::Schemas(
                        <Erc20 as odra::contract_def::HasEvents>::event_schemas()
                    );
                    let named_args = Option::<odra::RuntimeArgs>::None;
                    odra::odra_casper_wasm_env::host_functions::install_contract(
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
