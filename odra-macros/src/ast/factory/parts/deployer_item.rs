use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::{ast::deployer_utils::EpcSignature, utils, ModuleImplIR};

pub struct FactoryDeployImplItem {
    ident: syn::Ident,
    epc_fn: FactoryContractEpcFn,
}

impl TryFrom<&'_ ModuleImplIR> for FactoryDeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
       
        Ok(Self {
            ident: module.host_ref_ident()?,
            epc_fn: module.try_into()?,
        })
    }
}

impl ToTokens for FactoryDeployImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let epc_ty = utils::ty::entry_point_caller_provider();
        let ident = &self.ident;
        let epc_fn = &self.epc_fn;

        tokens.append_all(quote::quote! {
            impl #epc_ty for #ident {
                #epc_fn
            }
        });
    }
}

struct FactoryContractEpcFn {
    sig: EpcSignature,
    entry_points_expr: syn::Expr,
    caller: FactoryEntrypointCallerExpr
}

impl TryFrom<&'_ ModuleImplIR> for FactoryContractEpcFn {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let fun = module.factory_fn();
        let entry_point = utils::expr::new_entry_point(fun.name_str(), fun.raw_typed_args(), fun.is_payable());
        Ok(Self {
            sig: module.try_into()?,
            entry_points_expr: utils::expr::vec(entry_point),
            caller: module.try_into()?
        })
    }
}

impl ToTokens for FactoryContractEpcFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.sig;
        let entry_points_ident = utils::ident::entry_points();
        let caller = &self.caller;
        let entry_points_expr = &self.entry_points_expr;


        tokens.append_all(quote::quote! {
            #sig {
                let #entry_points_ident = #entry_points_expr;
                #caller
            }
        });
    }
}

#[derive(syn_derive::ToTokens)]
struct FactoryEntrypointCallerExpr {
    caller_expr: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for FactoryEntrypointCallerExpr {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
       
        Ok(Self {
            caller_expr: Self::entrypoint_caller(module)?

        })
    }
}

impl FactoryEntrypointCallerExpr {
    fn entrypoint_caller(_module: &ModuleImplIR) -> syn::Result<syn::Expr> {
        let env_ident = utils::ident::env();
        let entry_points_ident = utils::ident::entry_points();
        let contract_env_ident = utils::ident::contract_env();
        let call_def_ident = utils::ident::call_def();
        let ty_caller = utils::ty::entry_points_caller();

        Ok(parse_quote!(
            #ty_caller::new(#env_ident.clone(), #entry_points_ident, |#contract_env_ident, #call_def_ident| {
                if #call_def_ident.entry_point() == "factory" {
                    return Err(OdraError::VmError(
                        odra::VmError::Other(odra::prelude::String::from("Factory is not supported for this configuration."))
                    ));
                }
                Err(OdraError::VmError(odra::VmError::NoSuchMethod(odra::prelude::String::from(#call_def_ident.entry_point()))))
            })
        ))
    }
}
