use derive_try_from_ref::TryFromRef;
use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::{ast::deployer_utils::EpcSignature, utils, ModuleImplIR};

pub struct FactoryDeployImplItem {
    ident: syn::Ident,
    epc_fn: FactoryContractEpcFn
}

impl TryFrom<&'_ ModuleImplIR> for FactoryDeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ident: module.host_ref_ident()?,
            epc_fn: module.try_into()?
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

#[derive(TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
struct FactoryContractEpcFn {
    sig: EpcSignature,
    caller: FactoryEntrypointCallerExpr
}

impl ToTokens for FactoryContractEpcFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.sig;
        let entry_points_ident = utils::ident::entry_points();
        let vec = utils::expr::empty_vec();
        let caller = &self.caller;

        tokens.append_all(quote::quote! {
            #sig {
                let #entry_points_ident = #vec;
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
                Err(OdraError::VmError(odra::VmError::NoSuchMethod(odra::prelude::String::from(#call_def_ident.entry_point()))))
            })
        ))
    }
}
