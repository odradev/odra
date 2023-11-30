use super::parts_utils::{UsePreludeItem, UseSuperItem};
use crate::{
    ir::{FnIR, ModuleIR},
    utils
};
use derive_try_from::TryFromRef;
use quote::TokenStreamExt;
use syn::parse_quote;

#[derive(syn_derive::ToTokens)]
pub struct ExecPartsReexportItem {
    reexport_stmt: syn::Stmt
}

impl TryFrom<&'_ ModuleIR> for ExecPartsReexportItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let test_parts_ident = module.exec_parts_mod_ident()?;
        Ok(Self {
            reexport_stmt: parse_quote!(pub use #test_parts_ident::*;)
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct ExecPartsItem {
    parts_module: ExecPartsModuleItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    use_super: UseSuperItem,
    #[syn(in = brace_token)]
    use_prelude: UsePreludeItem,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    exec_functions: Vec<ExecFunctionItem>
}

impl TryFrom<&'_ ModuleIR> for ExecPartsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            parts_module: module.try_into()?,
            brace_token: Default::default(),
            use_prelude: UsePreludeItem,
            use_super: UseSuperItem,
            exec_functions: module
                .functions()
                .iter()
                .map(|f| (module, f))
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ExecFunctionItem {
    inline_attr: syn::Attribute,
    sig: ExecFnSignature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    args: Vec<syn::Stmt>,
    #[syn(in = braces)]
    init_contract_stmt: syn::Stmt,
    #[syn(in = braces)]
    return_stmt: syn::Stmt
}

impl TryFrom<(&'_ ModuleIR, &'_ FnIR)> for ExecFunctionItem {
    type Error = syn::Error;

    fn try_from(value: (&'_ ModuleIR, &'_ FnIR)) -> Result<Self, Self::Error> {
        let (module, func) = value;
        let fn_ident = func.name();
        let env_ident = utils::ident::env();
        let contract_ident = utils::ident::contract();
        let module_ident = module.module_ident()?;
        let fn_args = func.arg_names();

        let args = func
            .arg_names()
            .iter()
            .map(|ident| utils::stmt::get_named_arg(ident, &env_ident))
            .collect();

        let init_contract_stmt = match func.is_mut() {
            true => utils::stmt::new_mut_module(&contract_ident, &module_ident, &env_ident),
            false => utils::stmt::new_module(&contract_ident, &module_ident, &env_ident)
        };

        Ok(Self {
            inline_attr: utils::attr::inline(),
            sig: func.try_into()?,
            braces: Default::default(),
            args,
            init_contract_stmt,
            return_stmt: parse_quote!(return #contract_ident.#fn_ident( #(#fn_args),* );)
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ExecFnSignature {
    vis: syn::Visibility,
    fn_token: syn::token::Fn,
    ident: syn::Ident,
    #[syn(parenthesized)]
    paren: syn::token::Paren,
    #[syn(in = paren)]
    env_ident: syn::Ident,
    #[syn(in = paren)]
    colon_token: syn::token::Colon,
    #[syn(in = paren)]
    env_type: syn::Type,
    ret_ty: syn::ReturnType
}

impl TryFrom<&'_ FnIR> for ExecFnSignature {
    type Error = syn::Error;

    fn try_from(func: &'_ FnIR) -> Result<Self, Self::Error> {
        Ok(Self {
            vis: utils::syn::visibility_pub(),
            fn_token: Default::default(),
            ident: func.execute_name(),
            paren: Default::default(),
            env_ident: utils::ident::env(),
            colon_token: Default::default(),
            env_type: utils::ty::contract_env(),
            ret_ty: func.return_type()
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleIR)]
struct ExecPartsModuleItem {
    #[default]
    mod_token: syn::token::Mod,
    #[expr(item.exec_parts_mod_ident()?)]
    ident: syn::Ident
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{self, mock_module};

    #[test]
    fn test_parts() {
        let module = mock_module();
        let actual = ExecPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            mod __erc20_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_init(env: odra::ContractEnv) {
                    let total_supply = env.get_named_arg("total_supply");
                    let mut contract = Erc20::new(Rc::new(env));
                    return contract.init(total_supply);
                }

                #[inline]
                pub fn execute_total_supply(env: odra::ContractEnv) -> U256 {
                    let contract = Erc20::new(Rc::new(env));
                    return contract.total_supply();
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
