use super::{fn_utils, ref_utils};
use crate::{
    ir::{FnArgIR, FnIR, ModuleIR},
    utils
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_quote;

#[derive(syn_derive::ToTokens)]
pub struct DeployerInitSignature {
    fn_token: syn::token::Fn,
    init_token: syn::Ident,
    #[syn(parenthesized)]
    paren_token: syn::token::Paren,
    #[syn(in = paren_token)]
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    output: syn::ReturnType
}

impl TryFrom<&'_ ModuleIR> for DeployerInitSignature {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let host_ref_ident = module.host_ref_ident()?;
        let ty_host_env = utils::ty::host_env();
        let env = utils::ident::env();

        let mut inputs = module.constructor_args();
        inputs.insert(0, parse_quote!(#env: &#ty_host_env));

        Ok(Self {
            fn_token: Default::default(),
            init_token: utils::ident::init(),
            paren_token: Default::default(),
            inputs,
            output: parse_quote!(-> #host_ref_ident)
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct EntrypointCallerExpr {
    let_token: syn::token::Let,
    ident: syn::Ident,
    assign_token: syn::token::Eq,
    caller_expr: syn::Expr,
    semi_token: syn::token::Semi
}

impl TryFrom<&'_ ModuleIR> for EntrypointCallerExpr {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            let_token: Default::default(),
            ident: utils::ident::caller(),
            assign_token: Default::default(),
            caller_expr: Self::entrypoint_caller(module)?,
            semi_token: Default::default()
        })
    }
}

impl EntrypointCallerExpr {
    fn entrypoint_caller(module: &ModuleIR) -> Result<syn::Expr, syn::Error> {
        let env_ident = utils::ident::env();
        let contract_env_ident = utils::ident::contract_env();
        let call_def_ident = utils::ident::call_def();
        let ty_caller = utils::ty::entry_points_caller();

        let mut branches: Vec<CallerBranch> = module
            .functions()
            .iter()
            .map(|f| Ok(CallerBranch::Function(FunctionCallBranch::new(module, f)?)))
            .collect::<Result<_, syn::Error>>()?;
        branches.push(CallerBranch::Default(DefaultBranch));

        Ok(parse_quote!(
            #ty_caller::new(#env_ident.clone(), |#contract_env_ident, #call_def_ident| {
                match #call_def_ident.method() {
                    #(#branches)*
                }
            })
        ))
    }
}

#[derive(syn_derive::ToTokens)]
pub struct NewContractExpr {
    let_token: syn::token::Let,
    ident: syn::Ident,
    assign_token: syn::token::Eq,
    new_contract_expr: syn::Expr,
    semi_token: syn::token::Semi
}

impl TryFrom<&'_ ModuleIR> for NewContractExpr {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let module_str = module.module_str()?;
        let caller_ident = utils::ident::caller();
        let env_ident = utils::ident::env();
        let args = module
            .constructor()
            .map(|f| fn_utils::runtime_args_block(&f, ref_utils::insert_arg_stmt))
            .unwrap_or({
                let args = utils::expr::new_runtime_args();
                parse_quote!({#args})
            });

        let new_contract_expr = parse_quote!(
            #env_ident.new_contract(
                #module_str,
                Some(#args),
                Some(#caller_ident)
            )
        );

        Ok(Self {
            let_token: Default::default(),
            ident: utils::ident::address(),
            assign_token: Default::default(),
            new_contract_expr,
            semi_token: Default::default()
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct HostRefInstanceExpr {
    ident: syn::Ident,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    fields: syn::punctuated::Punctuated<syn::FieldValue, syn::Token![,]>
}

impl TryFrom<&'_ ModuleIR> for HostRefInstanceExpr {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let address_ident = utils::ident::address();
        let env_ident = utils::ident::env();
        let attached_value_ident = utils::ident::attached_value();
        let zero = utils::expr::u512_zero();

        let fields = parse_quote!(
            #address_ident,
            #env_ident: #env_ident.clone(),
            #attached_value_ident: #zero

        );
        Ok(Self {
            ident: module.host_ref_ident()?,
            braces: Default::default(),
            fields
        })
    }
}

#[derive(syn_derive::ToTokens)]
enum CallerBranch {
    Function(FunctionCallBranch),
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

impl<'a> FunctionCallBranch {
    pub fn new(module: &'a ModuleIR, func: &'a FnIR) -> Result<Self, syn::Error> {
        let call_stmt = Self::call_stmt(module, func)?;
        let result_expr = utils::expr::parse_bytes(&utils::ident::result());

        Ok(Self {
            function_name: func.name_str(),
            arrow_token: Default::default(),
            brace_token: Default::default(),
            call_stmt,
            result_expr
        })
    }

    fn read_call_def_arg_expr(arg: &FnArgIR) -> Result<syn::Expr, syn::Error> {
        let call_def_ident = utils::ident::call_def();
        let name = arg.name_str()?;
        Ok(parse_quote!(#call_def_ident.get(#name).expect("arg not found")))
    }

    fn call_stmt(module: &'a ModuleIR, func: &'a FnIR) -> Result<syn::Stmt, syn::Error> {
        let args = func
            .named_args()
            .iter()
            .map(Self::read_call_def_arg_expr)
            .collect::<Result<syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>, syn::Error>>(
            )?;

        let result_ident = utils::ident::result();
        let module_instance = module.module_instance_expr(utils::ident::contract_env())?;
        let function_ident = func.name();
        Ok(parse_quote!(let #result_ident = #module_instance.#function_ident(#args);))
    }
}

struct DefaultBranch;

impl ToTokens for DefaultBranch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote::quote!(_ => panic!("Unknown method")))
    }
}
