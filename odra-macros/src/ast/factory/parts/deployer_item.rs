use quote::{format_ident, ToTokens, TokenStreamExt};
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
        let mut entry_points = [module.factory_fn(), module.factory_upgrade_fn()]
            .iter()
            .map(|fun| {
                utils::expr::new_entry_point(
                    fun.name_str(),
                    fun.raw_typed_args(),
                    fun.is_payable(),
                    false
                )
            })
            .collect::<syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>>();
        // The batch upgrade arg is generic, so its type is given explicitly.
        let ty_entry_point = utils::ty::odra_entry_point();
        let ty_entry_point_arg = utils::ty::odra_entry_point_arg();
        let ty_batch_upgrade_args = utils::ty::batch_upgrade_args();
        let batch_upgrade_name =
            utils::expr::string_from(module.factory_batch_upgrade_fn().name_str());
        let args_name = utils::expr::string_from(String::from("args"));
        entry_points.push(parse_quote!(#ty_entry_point::new(
            #batch_upgrade_name,
            odra::prelude::vec![
                #ty_entry_point_arg::new::<#ty_batch_upgrade_args<odra::host::NoArgs>>(#args_name)
            ]
        )));
        Ok(Self {
            sig: module.try_into()?,
            entry_points_expr: utils::expr::vec(entry_points),
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

/// Runs the factory entry points with the helpers of `odra::host::factory`. They deploy and
/// upgrade the children natively, which only OdraVM supports; a factory compiled to Wasm runs
/// the `no_mangle` entry points instead.
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
    fn entrypoint_caller(module: &ModuleImplIR) -> syn::Result<syn::Expr> {
        let env_ident = utils::ident::env();
        let entry_points_ident = utils::ident::entry_points();
        let contract_env_ident = utils::ident::contract_env();
        let call_def_ident = utils::ident::call_def();
        let ty_caller = utils::ty::entry_points_caller();
        let ty_epc_provider = utils::ty::entry_point_caller_provider();

        let module_ident = module.module_ident()?;
        let module_str = module_ident.to_string();
        let child_str = module_str.strip_suffix("Factory").unwrap_or(&module_str);
        let child_host_ref = format_ident!("{}HostRef", child_str);
        let event_ident = format_ident!("{}ContractDeployed", module_str);

        let branches = [
            module.factory_fn(),
            module.factory_upgrade_fn(),
            module.factory_batch_upgrade_fn()
        ]
        .iter()
        .map(|fun| {
            let name = fun.name_str();
            let helper = fun.name();
            quote::quote! {
                #name => odra::host::factory::#helper(
                    &#contract_env_ident,
                    &#call_def_ident,
                    <#child_host_ref as #ty_epc_provider>::entry_points_caller,
                    |contract_name, contract_address| #event_ident {
                        contract_name,
                        contract_address
                    }
                ),
            }
        })
        .collect::<proc_macro2::TokenStream>();

        Ok(parse_quote!(
            #ty_caller::new(#env_ident.clone(), #entry_points_ident, |#contract_env_ident, #call_def_ident| {
                match #call_def_ident.entry_point() {
                    #branches
                    name => Err(OdraError::VmError(
                        odra::VmError::NoSuchMethod(odra::prelude::String::from(name))
                    ))
                }
            })
        ))
    }
}
