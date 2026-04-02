use crate::{
    ast::{
        parts_utils::{UsePreludeItem, UseSuperItem},
    },
    ir::{FnIR, ModuleImplIR},
    utils
};
use quote::{format_ident, ToTokens, TokenStreamExt};
use syn::{parse_quote, Ident};

pub(crate) struct FactoryExecPartsItem {
    module: Ident,
    use_super: UseSuperItem,
    use_prelude: UsePreludeItem,
    exec_functions: Vec<ExecFunctionItem>
}

impl TryFrom<&'_ ModuleImplIR> for FactoryExecPartsItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module: module.exec_parts_mod_ident()?,
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

impl ToTokens for FactoryExecPartsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.module;
        let missing_docs_attr = utils::attr::missing_docs();
        let use_prelude = &self.use_prelude;
        let use_super = &self.use_super;
        let fns = &self.exec_functions;

        tokens.append_all(quote::quote! {
            #missing_docs_attr
            mod #ident {
                #use_super
                #use_prelude

                #(#fns)*
            }
        });
    }
}

#[derive(syn_derive::ToTokens)]
struct ExecFunctionItem {
    inline_attr: syn::Attribute,
    vis: syn::Visibility,
    sig: syn::Signature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    fn_body: Box<dyn ToTokens>
}

impl TryFrom<(&'_ ModuleImplIR, &'_ FnIR)> for ExecFunctionItem {
    type Error = syn::Error;

    fn try_from(value: (&'_ ModuleImplIR, &'_ FnIR)) -> Result<Self, Self::Error> {
        let (_, func) = value;

        let fn_body: Box<dyn ToTokens> = if func.is_factory() {
            Box::new(quote::quote! {})
        } else {
            Box::new(ExecutableFnBodyItem::try_from(value)?)
        };
        Ok(Self {
            inline_attr: utils::attr::inline(),
            vis: utils::syn::visibility_pub(),
            sig: <&FnIR as Into<ExecFnSignature>>::into(func).try_into()?,
            braces: Default::default(),
            fn_body
        })
    }
}


#[derive(syn_derive::ToTokens)]
struct ExecutableFnBodyItem {
    env_rc_stmt: syn::Stmt,
    exec_env_stmt: Option<syn::Stmt>,
    non_reentrant_before_stmt: Option<ExecEnvStmt>,
    handle_attached_value_stmt: Option<ExecEnvStmt>,
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    args: Vec<syn::Stmt>,
    init_contract_stmt: syn::Stmt,
    call_contract_stmt: syn::Stmt,
    clear_attached_value_stmt: Option<ExecEnvStmt>,
    non_reentrant_after_stmt: Option<ExecEnvStmt>,
    return_stmt: syn::Stmt
}

impl TryFrom<(&'_ ModuleImplIR, &'_ FnIR)> for ExecutableFnBodyItem {
    type Error = syn::Error;

    fn try_from(value: (&'_ ModuleImplIR, &'_ FnIR)) -> Result<Self, Self::Error> {
        let (module, func) = value;
        let fn_ident = func.name();
        let result_ident = utils::ident::result();
        let env_rc_ident = utils::ident::env_rc();
        let env_ident = utils::ident::env();
        let exec_env_ident = utils::ident::exec_env();
        let exec_env_stmt =
            (func.is_payable() || func.is_non_reentrant() || func.has_args() || func.is_upgrader())
                .then(|| utils::stmt::new_execution_env(&exec_env_ident, &env_rc_ident));
        let contract_ident = utils::ident::contract();
        let module_ident = module.module_ident()?;
        let child_module_ident = format_ident!("{}", module_ident.to_string().strip_suffix("Factory").unwrap_or(&module_ident.to_string()));

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
            true => utils::stmt::new_mut_module(&contract_ident, &child_module_ident, &env_rc_ident),
            false => utils::stmt::new_module(&contract_ident, &child_module_ident, &env_rc_ident)
        };

        Ok(Self {
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

struct ExecFnSignature {
    ident: syn::Ident,
    ret_ty: syn::ReturnType
}

impl From<&'_ FnIR> for ExecFnSignature {
    fn from(func: &'_ FnIR) -> Self {
        Self {
            ident: func.execute_name(),
            ret_ty: if func.is_factory() {
                parse_quote!()
            } else {
                func.return_type()
            }
        }
    }
}

impl TryInto<syn::Signature> for ExecFnSignature {
    type Error = syn::Error;

    fn try_into(self) -> Result<syn::Signature, Self::Error> {
        let tokens = self.into_token_stream();
        let sig: syn::Signature = parse_quote!(#tokens);
        Ok(sig)
    }
}

impl ToTokens for ExecFnSignature {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let env_ident = utils::ident::env();
        let env_type = utils::ty::contract_env();
        let ret_ty = &self.ret_ty;

        tokens.append_all(quote::quote! {
            fn #ident(#env_ident: #env_type) #ret_ty
        });
    }
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
        let module = mock::module_factory_impl();
        let actual = FactoryExecPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[allow(missing_docs)]
            mod __erc20_factory_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_init(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    let value = exec_env.get_named_arg::<u32>("value");
                    let mut contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.init(value);
                    return result;
                }

                #[inline]
                pub fn execute_total_supply(env: odra::ContractEnv) -> U256 {
                    let env_rc = Rc::new(env);
                    let contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.total_supply();
                    return result;
                }

                #[inline]
                pub fn execute_pay_to_mint(env: odra::ContractEnv) {
                    let env_rc = Rc::new(env);
                    let exec_env = odra::ExecutionEnv::new(env_rc.clone());
                    exec_env.handle_attached_value();
                    let mut contract = <Erc20 as Module>::new(env_rc);
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
                    let msg = exec_env.get_named_arg::<Maybe<String>>("msg");
                    let mut contract = <Erc20 as Module>::new(env_rc);
                    let result = contract.approve(&to, &amount, msg);
                    exec_env.non_reentrant_after();
                    return result;
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
