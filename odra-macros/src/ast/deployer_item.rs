use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated};

use crate::{
    ast::Target,
    ir::{FnArgIR, ModuleImplIR},
    utils
};

/// Emits what a test needs to deploy the contract on a host: the `XxxInitArgs` and
/// `XxxUpgradeArgs` structs when `init` and `upgrade` take arguments, and
/// `impl EntryPointsCallerProvider for XxxHostRef`, which lists the entry points and dispatches
/// a call by name.
pub struct DeployerItem {
    init_args: Option<ArgsStructItem>,
    upgrade_args: Option<ArgsStructItem>,
    caller_provider_impl: CallerProviderImplItem
}

impl DeployerItem {
    pub(crate) fn try_new(module: &ModuleImplIR, kind: Target) -> syn::Result<Self> {
        let module_name = module.module_str();
        let host_ref_ident = module.host_ref_ident();
        match kind {
            Target::Module => Ok(Self {
                init_args: match module.constructor() {
                    Some(f) if f.has_args() => Some(ArgsStructItem {
                        role: ArgsRole::Init,
                        module_name: module_name.clone(),
                        ident: module.init_args_ident(),
                        fields: f.args().iter().map(FnArgIR::name_and_ty).collect()
                    }),
                    _ => None
                },
                upgrade_args: match module.upgrader() {
                    Some(f) if f.has_args() => Some(ArgsStructItem {
                        role: ArgsRole::Upgrade,
                        module_name,
                        ident: module.upgrade_args_ident(),
                        fields: f.args().iter().map(FnArgIR::name_and_ty).collect()
                    }),
                    _ => None
                },
                caller_provider_impl: CallerProviderImplItem {
                    host_ref_ident,
                    entry_points: module
                        .functions()
                        .iter()
                        .map(|f| {
                            new_entry_point(
                                f.name_str(),
                                f.args(),
                                f.is_payable(),
                                f.is_offchain()
                            )
                        })
                        .collect(),
                    branches: CallerBranches::ExecParts {
                        exec_parts_mod_ident: module.exec_parts_mod_ident(),
                        functions: module
                            .functions()
                            .iter()
                            .map(|f| (f.name_str(), f.execute_name()))
                            .collect()
                    }
                }
            }),
            Target::Factory => {
                let factory_fns = [
                    module.factory_fn(),
                    module.factory_upgrade_fn(),
                    module.factory_batch_upgrade_fn()
                ];
                let [new_contract_fn, upgrade_fn, batch_upgrade_fn] = &factory_fns;
                // The batch upgrade argument is generic, so its type is given explicitly.
                let batch_upgrade_name = batch_upgrade_fn.name_str();
                let batch_upgrade_entry_point: syn::Expr = syn::parse_quote!(odra::entry_point_callback::EntryPoint::new(
                    odra::prelude::string::String::from(#batch_upgrade_name),
                    odra::prelude::vec![
                        odra::entry_point_callback::Argument::new::<odra::args::BatchUpgradeArgs<odra::host::NoArgs>>(odra::prelude::string::String::from("args"))
                    ]
                ));
                Ok(Self {
                    init_args: None,
                    upgrade_args: None,
                    caller_provider_impl: CallerProviderImplItem {
                        host_ref_ident,
                        entry_points: [new_contract_fn, upgrade_fn]
                            .into_iter()
                            .map(|f| {
                                new_entry_point(
                                    f.name_str(),
                                    f.args(),
                                    f.is_payable(),
                                    false
                                )
                            })
                            .chain([batch_upgrade_entry_point])
                            .collect(),
                        branches: CallerBranches::FactoryHelpers {
                            child_host_ref_ident: format_ident!(
                                "{}HostRef",
                                module.child_module_ident()
                            ),
                            event_ident: format_ident!("{}ContractDeployed", module.module_ident()),
                            helpers: factory_fns.iter().map(|f| (f.name_str(), f.name())).collect()
                        }
                    }
                })
            }
        }
    }
}

impl TryFrom<&ModuleImplIR> for DeployerItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, Target::Module)
    }
}

impl ToTokens for DeployerItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            init_args,
            upgrade_args,
            caller_provider_impl
        } = self;
        tokens.extend(quote! {
            #init_args
            #upgrade_args
            #caller_provider_impl
        });
    }
}

enum ArgsRole {
    Init,
    Upgrade
}

/// Emits `pub struct XxxInitArgs { pub a: A, .. }` deriving `IntoRuntimeArgs`, and its empty
/// `impl odra::host::InitArgs` (or the `UpgradeArgs` pair): the typed arguments a test passes
/// to deploy or upgrade the contract.
struct ArgsStructItem {
    role: ArgsRole,
    module_name: String,
    ident: Ident,
    fields: Vec<(String, syn::Type)>
}

impl ToTokens for ArgsStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let (doc, ty_trait) = match self.role {
            ArgsRole::Init => (
                format!(" [{}] contract constructor arguments.", self.module_name),
                quote!(odra::host::InitArgs)
            ),
            ArgsRole::Upgrade => (
                format!(" [{}] contract upgrade arguments.", self.module_name),
                quote!(odra::host::UpgradeArgs)
            )
        };
        let fields = self.fields.iter().map(|(name, ty)| {
            let name = format_ident!("{}", name);
            let ty = utils::ty::unreferenced_ty(ty);
            quote!(pub #name: #ty)
        });

        tokens.extend(quote! {
            #[allow(missing_docs)]
            #[doc = #doc]
            #[derive(odra::IntoRuntimeArgs)]
            pub struct #ident {
                #(#fields),*
            }

            impl #ty_trait for #ident {}
        });
    }
}

/// Emits `impl EntryPointsCallerProvider for XxxHostRef`: `entry_points_caller()` lists the
/// entry points with their arguments and dispatches a call by its entry point name.
struct CallerProviderImplItem {
    host_ref_ident: Ident,
    entry_points: Vec<syn::Expr>,
    branches: CallerBranches
}

/// How a dispatched call is run.
enum CallerBranches {
    /// A module runs `__xxx_exec_parts::execute_<name>` and serializes its result.
    ExecParts {
        exec_parts_mod_ident: Ident,
        /// Entry point name and the `execute_` function of each.
        functions: Vec<(String, Ident)>
    },
    /// A factory runs `odra::host::factory::<name>`, which deploys or upgrades a child with the
    /// child's entry points caller and emits `XxxContractDeployed`.
    FactoryHelpers {
        child_host_ref_ident: Ident,
        event_ident: Ident,
        /// Entry point name and the helper of each.
        helpers: Vec<(String, Ident)>
    }
}

impl ToTokens for CallerProviderImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let host_ref_ident = &self.host_ref_ident;
        let entry_points_exprs = &self.entry_points;
        let contract_env = utils::ident::contract_env();
        let call_def = utils::ident::call_def();
        let result = utils::ident::result();

        let branches = match &self.branches {
            CallerBranches::ExecParts {
                exec_parts_mod_ident,
                functions
            } => functions
                .iter()
                .map(|(name, execute_fn)| {
                    quote! {
                        #name => {
                            let #result = #exec_parts_mod_ident::#execute_fn(#contract_env);
                            odra::utils::try_serialize(&#result)
                        }
                    }
                })
                .collect::<TokenStream>(),
            CallerBranches::FactoryHelpers {
                child_host_ref_ident,
                event_ident,
                helpers
            } => helpers
                .iter()
                .map(|(name, helper)| {
                    quote! {
                        #name => odra::host::factory::#helper(
                            &#contract_env,
                            &#call_def,
                            <#child_host_ref_ident as odra::host::EntryPointsCallerProvider>::entry_points_caller,
                            |contract_name, contract_address| #event_ident {
                                contract_name,
                                contract_address
                            }
                        ),
                    }
                })
                .collect::<TokenStream>()
        };

        tokens.extend(quote! {
            impl odra::host::EntryPointsCallerProvider for #host_ref_ident {
                fn entry_points_caller() -> odra::entry_point_callback::EntryPointsCaller {
                    let entry_points = odra::prelude::vec![#(#entry_points_exprs),*];
                    odra::entry_point_callback::EntryPointsCaller::new(entry_points, |#contract_env, #call_def| {
                        match #call_def.entry_point() {
                            #branches
                            name => Err(OdraError::VmError(
                                odra::VmError::NoSuchMethod(odra::prelude::String::from(name))
                            ))
                        }
                    })
                }
            }
        });
    }
}

/// `odra::entry_point_callback::EntryPoint::new[_payable|_offchain](name, args)`.
fn new_entry_point(
    name: String,
    args: &[FnArgIR],
    is_payable: bool,
    is_offchain: bool
) -> syn::Expr {
    let name = string_from(&name);
    let args_stream = args
        .iter()
        .map(new_entry_point_arg)
        .collect::<Punctuated<_, syn::Token![,]>>();
    let args_vec: syn::Expr = parse_quote!(odra::prelude::vec![#args_stream]);
    if is_offchain {
        parse_quote!(odra::entry_point_callback::EntryPoint::new_offchain(#name, #args_vec))
    } else if is_payable {
        parse_quote!(odra::entry_point_callback::EntryPoint::new_payable(#name, #args_vec))
    } else {
        parse_quote!(odra::entry_point_callback::EntryPoint::new(#name, #args_vec))
    }
}
fn new_entry_point_arg(arg: &FnArgIR) -> syn::Expr {
    let arg_ty = utils::ty::unreferenced_ty(arg.ty());
    let name = string_from(&arg.name_str());
    parse_quote!(odra::entry_point_callback::Argument::new::<#arg_ty>(#name))
}

fn string_from(string: &str) -> syn::Expr {
    parse_quote!(odra::prelude::string::String::from(#string))
}

#[cfg(test)]
mod tests;
