use quote::TokenStreamExt;
use syn::parse_quote;

use crate::{
    ast::fn_utils,
    ir::{FnIR, ModuleIR},
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
    call_fn: CallFnItem
}

impl TryFrom<&'_ ModuleIR> for WasmPartsModuleItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let module_str = module.module_str()?;
        let ident = module.wasm_parts_mod_ident()?;
        Ok(Self {
            attrs: vec![utils::attr::wasm32(), utils::attr::odra_module(&module_str)],
            mod_token: Default::default(),
            ident,
            braces: Default::default(),
            use_super: UseSuperItem,
            use_prelude: UsePreludeItem,
            entry_points_fn: module.try_into()?,
            call_fn: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct EntryPointsFnItem {
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

impl TryFrom<&'_ ModuleIR> for EntryPointsFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let ty_entry_points = utils::ty::entry_points();
        let ident_entry_points = utils::ident::entry_points();
        let expr_entry_points = utils::expr::new_entry_points();

        Ok(Self {
            sig: parse_quote!(fn #ident_entry_points() -> #ty_entry_points),
            braces: Default::default(),
            var_declaration: parse_quote!(let mut #ident_entry_points = #expr_entry_points;),
            items: module
                .functions()
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

impl TryFrom<&'_ ModuleIR> for CallFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
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
        let expr_new_schemas = utils::expr::new_schemas();
        let expr_install = utils::expr::install_contract(
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
            install_contract_stmt: parse_quote!(#expr_install;)
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
        let var_ident = utils::ident::entry_points();
        let fn_ident = utils::ident::add_entry_point();
        Ok(Self {
            var_ident,
            dot_token: Default::default(),
            fn_ident,
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
        let module = test_utils::mock_module();
        let actual = WasmPartsModuleItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
            mod __erc20_wasm_parts {
                use super::*;
                use odra::prelude::*;

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
                }

                #[no_mangle]
                fn call() {
                    let schemas = odra::casper_event_standard::Schemas::new();
                    let named_args = Some({
                        let mut named_args = odra::RuntimeArgs::new();
                        let _ = named_args.insert(
                            "total_supply",
                            odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_named_arg("total_supply")
                        );
                        named_args
                    });
                    odra::odra_casper_wasm_env::host_functions::install_contract(
                        entry_points(),
                        schemas,
                        named_args
                    );
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
