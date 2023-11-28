use quote::TokenStreamExt;
use syn::parse_quote;

use crate::{ir::{ModuleIR, FnIR}, utils};

use super::parts_utils::{UseSuperItem, UsePreludeItem};

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
    entry_points_fn: EntryPointsFnItem
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
            entry_points_fn: module.try_into()?
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
            sig: parse_quote!(fn entry_points() -> #ty_entry_points),
            braces: Default::default(), 
            var_declaration: parse_quote!(let mut #ident_entry_points = #expr_entry_points;),
            items: module.functions().iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>()?,
            ret: parse_quote!(#ident_entry_points),
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
            semi_token: Default::default(),
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
        let param_parameters = Self::param_parameters(func);
        let param_ret_ty = Self::param_ret_ty(func);
        let param_access = Self::param_access(func);

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


impl NewEntryPointItem {
    fn param_parameters(func: &FnIR) -> syn::Expr {
        let params = func.named_args()
            .iter()
            .map(|arg| arg.name_and_ty())
            .filter_map(|result| match result {
                Ok(data) => Some(data),
                Err(_) => None,
            })
            .map(|(name, ty)| utils::expr::new_parameter(name, ty))
            .collect::<Vec<_>>();
        parse_quote!(vec![#(#params),*])
    }

    fn param_access(func: &FnIR) -> syn::Expr {
        match func.is_constructor() {
            true => utils::expr::entry_point_group("constructor_group"),
            false => utils::expr::entry_point_public()
        }
    }

    fn param_ret_ty(func: &FnIR) -> syn::Expr {
        match func.return_type() {
            syn::ReturnType::Default => utils::expr::unit_cl_type(),
            syn::ReturnType::Type(_, ty) => utils::expr::as_cl_type(&ty),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use super::WasmPartsModuleItem;

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
            }
        };
      
        test_utils::assert_eq(actual, expected);
    }
}