use super::parts_utils::{UsePreludeItem, UseSuperItem};
use crate::{
    ir::{FnIR, ModuleImplIR},
    utils
};
use derive_try_from_ref::TryFromRef;
use quote::TokenStreamExt;
use syn::parse_quote;

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

impl TryFrom<&'_ ModuleImplIR> for ExecPartsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            parts_module: module.try_into()?,
            brace_token: Default::default(),
            use_prelude: UsePreludeItem,
            use_super: UseSuperItem,
            exec_functions: module
                .functions()?
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
    env_rc_stmt: syn::Stmt,
    #[syn(in = braces)]
    exec_env_stmt: Option<syn::Stmt>,
    #[syn(in = braces)]
    non_reentrant_before_stmt: Option<ExecEnvStmt>,
    #[syn(in = braces)]
    handle_attached_value_stmt: Option<ExecEnvStmt>,
    #[syn(in = braces)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    args: Vec<syn::Stmt>,
    #[syn(in = braces)]
    init_contract_stmt: syn::Stmt,
    #[syn(in = braces)]
    call_contract_stmt: syn::Stmt,
    #[syn(in = braces)]
    clear_attached_value_stmt: Option<ExecEnvStmt>,
    #[syn(in = braces)]
    non_reentrant_after_stmt: Option<ExecEnvStmt>,
    #[syn(in = braces)]
    return_stmt: syn::Stmt
}

impl TryFrom<(&'_ ModuleImplIR, &'_ FnIR)> for ExecFunctionItem {
    type Error = syn::Error;

    fn try_from(value: (&'_ ModuleImplIR, &'_ FnIR)) -> Result<Self, Self::Error> {
        let (module, func) = value;
        let fn_ident = func.name();
        let result_ident = utils::ident::result();
        let env_rc_ident = utils::ident::env_rc();
        let env_ident = utils::ident::env();
        let exec_env_ident = utils::ident::exec_env();
        let exec_env_stmt = (func.is_payable() || func.is_non_reentrant() || func.has_args())
            .then(|| utils::stmt::new_execution_env(&exec_env_ident, &env_rc_ident));
        let contract_ident = utils::ident::contract();
        let module_ident = module.module_ident()?;
        let fn_args = func
            .named_args()
            .iter()
            .map(|arg| {
                let ident = arg.name()?;
                let ref_token = arg.is_ref().then(|| quote::quote!(&));
                let expr: syn::Expr = parse_quote!(#ref_token #ident);
                Ok(expr)
            })
            .collect::<syn::Result<syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>>>()?;

        let args = func
            .named_args()
            .iter()
            .map(|arg| {
                let ty = utils::ty::unreferenced_ty(&arg.ty()?);
                Ok(utils::stmt::get_named_arg(
                    &arg.name()?,
                    &exec_env_ident,
                    &ty
                ))
            })
            .collect::<syn::Result<Vec<syn::Stmt>>>()?;

        let init_contract_stmt = match func.is_mut() {
            true => utils::stmt::new_mut_module(&contract_ident, &module_ident, &env_rc_ident),
            false => utils::stmt::new_module(&contract_ident, &module_ident, &env_rc_ident)
        };

        Ok(Self {
            inline_attr: utils::attr::inline(),
            sig: func.try_into()?,
            braces: Default::default(),
            env_rc_stmt: utils::stmt::new_rc(&env_rc_ident, &env_ident),
            exec_env_stmt,
            non_reentrant_before_stmt: func
                .is_non_reentrant()
                .then(ExecEnvStmt::non_reentrant_before),
            handle_attached_value_stmt: func.is_payable().then(ExecEnvStmt::handle_attached_value),
            args,
            init_contract_stmt,
            call_contract_stmt: parse_quote!(let #result_ident = #contract_ident.#fn_ident(#fn_args);),
            clear_attached_value_stmt: func.is_payable().then(ExecEnvStmt::clear_attached_value),
            non_reentrant_after_stmt: func
                .is_non_reentrant()
                .then(ExecEnvStmt::non_reentrant_after),
            return_stmt: parse_quote!(return #result_ident;)
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
#[source(ModuleImplIR)]
#[err(syn::Error)]
struct ExecPartsModuleItem {
    #[default]
    mod_token: syn::token::Mod,
    #[expr(input.exec_parts_mod_ident()?)]
    ident: syn::Ident
}

#[derive(syn_derive::ToTokens)]
struct ExecEnvStmt {
    ident: syn::Ident,
    dot_token: syn::token::Dot,
    call_expr: syn::ExprCall,
    semi_token: syn::token::Semi
}

impl ExecEnvStmt {
    fn new(call_expr: syn::ExprCall) -> Self {
        Self {
            ident: utils::ident::exec_env(),
            dot_token: Default::default(),
            call_expr,
            semi_token: Default::default()
        }
    }

    fn non_reentrant_before() -> Self {
        Self::new(parse_quote!(non_reentrant_before()))
    }

    fn non_reentrant_after() -> Self {
        Self::new(parse_quote!(non_reentrant_after()))
    }

    fn handle_attached_value() -> Self {
        Self::new(parse_quote!(handle_attached_value()))
    }

    fn clear_attached_value() -> Self {
        Self::new(parse_quote!(clear_attached_value()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{self, mock};

    #[test]
    fn test_parts() {
        let module = mock::module_impl();
        let actual = ExecPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            mod __erc20_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_init(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    let total_supply = exec_env.get_named_arg::<Option<U256>>("total_supply");
                    let mut contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.init(total_supply);
                    return result;
                }

                #[inline]
                pub fn execute_total_supply(env: odra::ContractEnv) -> U256 {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.total_supply();
                    return result;
                }

                #[inline]
                pub fn execute_pay_to_mint(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    exec_env.handle_attached_value();
                    let mut contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.pay_to_mint();
                    exec_env.clear_attached_value();
                    return result;
                }

                #[inline]
                pub fn execute_approve(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    exec_env.non_reentrant_before();
                    let to = exec_env.get_named_arg::<Address>("to");
                    let amount = exec_env.get_named_arg::<U256>("amount");
                    let mut contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.approve(&to, &amount);
                    exec_env.non_reentrant_after();
                    return result;
                }

                #[inline]
                pub fn execute_airdrop(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    let to = exec_env.get_named_arg::<odra::prelude::vec::Vec<Address>>("to");
                    let amount = exec_env.get_named_arg::<U256>("amount");
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.airdrop(&to, &amount);
                    return result;
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_trait_impl_parts() {
        let module = mock::module_trait_impl();
        let actual = ExecPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            mod __erc20_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_total_supply(env: odra::ContractEnv) -> U256 {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.total_supply();
                    return result;
                }

                #[inline]
                pub fn execute_pay_to_mint(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    exec_env.handle_attached_value();
                    let mut contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.pay_to_mint();
                    exec_env.clear_attached_value();
                    return result;
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_delegated_parts() {
        let module = mock::module_delegation();
        let actual = ExecPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            mod __erc20_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_total_supply(env: odra::ContractEnv) -> U256 {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.total_supply();
                    return result;
                }

                #[inline]
                pub fn execute_get_owner(env: odra::ContractEnv) -> Address {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.get_owner();
                    return result;
                }

                #[inline]
                pub fn execute_set_owner(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    let new_owner = exec_env.get_named_arg::<Address>("new_owner");
                    let mut contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.set_owner(new_owner);
                    return result;
                }

                #[inline]
                pub fn execute_name(env: odra::ContractEnv) -> String {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.name();
                    return result;
                }

                #[inline]
                pub fn execute_symbol(env: odra::ContractEnv) -> String {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as odra::module::Module>::new(env_rc);
                    let result = contract.symbol();
                    return result;
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
