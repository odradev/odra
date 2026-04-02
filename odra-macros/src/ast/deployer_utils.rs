use crate::{
    ir::{FnIR, ModuleImplIR},
    utils
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_quote;
use syn::punctuated::Punctuated;

pub struct EntrypointsInitExpr {
    value_expr: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for EntrypointsInitExpr {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let entry_points = module
            .functions()?
            .iter()
            .map(|f| utils::expr::new_entry_point(f.name_str(), f.raw_typed_args(), f.is_payable()))
            .collect::<Punctuated<_, syn::Token![,]>>();

        Ok(Self {
            value_expr: utils::expr::vec(entry_points)
        })
    }
}

impl ToTokens for EntrypointsInitExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = utils::ident::entry_points();
        let value_expr = &self.value_expr;
        tokens.extend(quote::quote! {
            let #ident = #value_expr;
        });
    }
}

#[derive(syn_derive::ToTokens)]
pub struct EpcSignature {
    fn_token: syn::token::Fn,
    epc_token: syn::Ident,
    #[syn(parenthesized)]
    paren_token: syn::token::Paren,
    #[syn(in = paren_token)]
    input: syn::FnArg,
    output: syn::ReturnType
}

impl TryFrom<&'_ ModuleImplIR> for EpcSignature {
    type Error = syn::Error;

    fn try_from(_: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let epc_ident = utils::ty::entry_points_caller();
        let ty_host_env = utils::ty::host_env();
        let env = utils::ident::env();

        let input = parse_quote!(#env: &#ty_host_env);

        Ok(Self {
            fn_token: Default::default(),
            epc_token: utils::ident::epc(),
            paren_token: Default::default(),
            input,
            output: utils::misc::ret_ty(&epc_ident)
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct EntrypointCallerExpr {
    caller_expr: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for EntrypointCallerExpr {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            caller_expr: Self::entrypoint_caller(module)?
        })
    }
}

impl EntrypointCallerExpr {
    fn entrypoint_caller(module: &ModuleImplIR) -> syn::Result<syn::Expr> {
        let env_ident = utils::ident::env();
        let entry_points_ident = utils::ident::entry_points();
        let contract_env_ident = utils::ident::contract_env();
        let call_def_ident = utils::ident::call_def();
        let ty_caller = utils::ty::entry_points_caller();

        let mut branches: Vec<CallerBranch> = module
            .functions()?
            .iter()
            .map(|f| FunctionCallBranch::try_from((module, f)))
            .map(|r| r.map(|b| CallerBranch::Function(Box::new(b))))
            .collect::<syn::Result<_>>()?;
        branches.push(CallerBranch::Default(DefaultBranch));

        Ok(parse_quote!(
            #ty_caller::new(#env_ident.clone(), #entry_points_ident, |#contract_env_ident, #call_def_ident| {
                match #call_def_ident.entry_point() {
                    #(#branches)*
                }
            })
        ))
    }
}


#[derive(syn_derive::ToTokens)]
enum CallerBranch {
    Function(Box<FunctionCallBranch>),
    Default(DefaultBranch)
}

#[derive(syn_derive::ToTokens)]
struct FunctionCallBranch {
    function_name: String,
    arrow_token: syn::token::FatArrow,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    call_stmt: syn::Stmt,
    #[syn(in = brace_token)]
    result_expr: syn::Expr
}

impl TryFrom<(&'_ ModuleImplIR, &'_ FnIR)> for FunctionCallBranch {
    type Error = syn::Error;

    fn try_from(value: (&'_ ModuleImplIR, &'_ FnIR)) -> Result<Self, Self::Error> {
        let (module, func) = value;
        Ok(Self {
            function_name: func.name_str(),
            arrow_token: Default::default(),
            brace_token: Default::default(),
            call_stmt: Self::call_stmt(module, func)?,
            result_expr: utils::expr::parse_bytes(&utils::ident::result())
        })
    }
}

impl<'a> FunctionCallBranch {
    fn call_stmt(module: &'a ModuleImplIR, func: &'a FnIR) -> syn::Result<syn::Stmt> {
        let result_ident = utils::ident::result();
        let function_ident = func.execute_name();
        let contract_env_ident = utils::ident::contract_env();
        let exec_parts_ident = module.exec_parts_mod_ident()?;
        Ok(
            parse_quote!(let #result_ident = #exec_parts_ident::#function_ident(#contract_env_ident);)
        )
    }
}

struct DefaultBranch;

impl ToTokens for DefaultBranch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote::quote!(name => Err(OdraError::VmError(
            odra::VmError::NoSuchMethod(odra::prelude::String::from(name))
        ))))
    }
}
