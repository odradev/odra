use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{contract_ref_item::WithTokensFnItem, ref_utils, Target},
    ir::ModuleImplIR
};

/// Emits `pub struct XxxHostRef { address, env, attached_value }`, the handle a test holds on a
/// deployed contract.
pub struct HostRefStructItem {
    module_name: String,
    ref_ident: Ident
}

impl TryFrom<&ModuleImplIR> for HostRefStructItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_name: module.module_str(),
            ref_ident: module.host_ref_ident()
        })
    }
}

impl ToTokens for HostRefStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let doc = format!(" [{}] Host Ref.", self.module_name);

        tokens.extend(quote! {
            #[doc = #doc]
            pub struct #ref_ident {
                address: odra::prelude::Address,
                env: odra::host::HostEnv,
                attached_value: odra::casper_types::U512
            }
        });
    }
}

/// Emits `impl odra::host::HostRef for XxxHostRef`: `new`, `with_tokens`, `address`, `env`,
/// `get_event` and `last_call`.
pub struct HostRefTraitImplItem {
    ref_ident: Ident,
    /// `new` registers the contract in the host env (`HostEnv::register_contract_ref`). An
    /// external contract (a trait) has no code to register.
    register: bool
}

impl TryFrom<&ModuleImplIR> for HostRefTraitImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ref_ident: module.host_ref_ident(),
            register: matches!(module, ModuleImplIR::Impl(_))
        })
    }
}

impl ToTokens for HostRefTraitImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let with_tokens_fn = WithTokensFnItem;

        let register = self.register.then(|| {
            quote! {
                env.register_contract_ref(
                    address,
                    <Self as odra::contract_def::HasIdent>::ident(),
                    <Self as odra::host::EntryPointsCallerProvider>::entry_points_caller
                );
            }
        });

        tokens.extend(quote! {
            impl odra::host::HostRef for #ref_ident {
                fn new(address: odra::prelude::Address, env: odra::host::HostEnv) -> Self {
                    #register
                    Self {
                        address,
                        env,
                        attached_value: Default::default()
                    }
                }

                #with_tokens_fn

                fn contract_address(&self) -> odra::prelude::Address {
                    self.address
                }

                fn env(&self) -> &odra::host::HostEnv {
                    &self.env
                }

                fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                where
                    T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance
                {
                    self.env.get_event(self, index)
                }

                fn last_call(&self) -> odra::ContractCallResult {
                    self.env.last_call_result(self.address)
                }
            }
        });
    }
}

/// Emits `impl odra::contract_def::HasIdent for XxxHostRef`: the host ref reports the same
/// ident and contract name as the module, so `InstallConfig::new::<XxxHostRef>()` and `::<Xxx>()`
/// agree. `contract_name` is called fully qualified, so an entry point of the same name cannot
/// shadow it.
pub struct HasIdentTraitImplItem {
    module_ident: Ident,
    ref_ident: Ident
}

impl TryFrom<&ModuleImplIR> for HasIdentTraitImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident(),
            ref_ident: module.host_ref_ident()
        })
    }
}

impl ToTokens for HasIdentTraitImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let ref_ident = &self.ref_ident;

        tokens.extend(quote! {
            impl odra::contract_def::HasIdent for #ref_ident {
                fn ident() -> odra::prelude::string::String {
                    #module_ident::ident()
                }
                fn contract_name() -> odra::prelude::string::String {
                    <#module_ident as odra::contract_def::HasIdent>::contract_name()
                }
            }
        });
    }
}

/// Emits one `impl XxxHostRef { .. }` (or `impl Trait for XxxHostRef { .. }`) holding the given
/// functions. The host ref has three: the entry points as written, their `try_` versions
/// returning `OdraResult`, and for a mutating entry point with a result, a `_no_ret` version.
struct HostRefFnsImplItem {
    trait_path: Option<syn::Path>,
    ref_ident: Ident,
    functions: Vec<syn::ItemFn>
}

impl ToTokens for HostRefFnsImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let functions = &self.functions;
        let for_trait = self.trait_path.as_ref().map(|t| quote!(#t for));

        tokens.extend(quote! {
            impl #for_trait #ref_ident {
                #(#functions)*
            }
        });
    }
}

/// Emits the whole host ref of a module: the struct, `impl HostRef`, and the impl blocks with
/// the entry points, their `try_` versions and the `_no_ret` versions. A factory's host ref has
/// the three factory functions and their `try_` versions instead.
pub struct HostRefItem {
    struct_item: HostRefStructItem,
    trait_impl_item: HostRefTraitImplItem,
    fns_impl_item: HostRefFnsImplItem,
    try_fns_impl_item: HostRefFnsImplItem,
    mut_no_ret_fns_impl_item: Option<HostRefFnsImplItem>
}

impl HostRefItem {
    pub(crate) fn try_new(module: &ModuleImplIR, target: Target) -> syn::Result<Self> {
        let ref_ident = module.host_ref_ident();
        let fns_impl = |trait_path, functions| HostRefFnsImplItem {
            trait_path,
            ref_ident: ref_ident.clone(),
            functions
        };
        let (fns_impl_item, try_fns_impl_item, mut_no_ret_fns_impl_item) = match target {
            Target::Module => {
                let is_trait_impl = module.is_trait_impl();
                let functions = module.functions();
                let mut_ret_functions = module.host_mut_ret_functions();
                (
                    fns_impl(
                        module.impl_trait_path(),
                        functions
                            .iter()
                            .map(|f| ref_utils::host_function_item(f, is_trait_impl))
                            .collect()
                    ),
                    fns_impl(
                        None,
                        functions
                            .iter()
                            .map(ref_utils::host_try_function_item)
                            .collect()
                    ),
                    (!mut_ret_functions.is_empty()).then(|| {
                        fns_impl(
                            None,
                            mut_ret_functions
                                .into_iter()
                                .flat_map(|f| {
                                    [
                                        ref_utils::host_mut_no_ret_function_item(f),
                                        ref_utils::host_mut_try_no_ret_function_item(f)
                                    ]
                                })
                                .collect()
                        )
                    })
                )
            }
            Target::Factory => {
                let functions = [
                    module.factory_fn(),
                    module.factory_upgrade_fn(),
                    module.factory_batch_upgrade_fn()
                ];
                (
                    fns_impl(
                        None,
                        functions
                            .iter()
                            .map(|f| ref_utils::host_function_item(f, false))
                            .collect()
                    ),
                    fns_impl(
                        None,
                        functions
                            .iter()
                            .map(ref_utils::factory_try_function_item)
                            .collect()
                    ),
                    None
                )
            }
        };

        Ok(Self {
            struct_item: HostRefStructItem::try_from(module)?,
            trait_impl_item: HostRefTraitImplItem::try_from(module)?,
            fns_impl_item,
            try_fns_impl_item,
            mut_no_ret_fns_impl_item
        })
    }
}

impl TryFrom<&ModuleImplIR> for HostRefItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, Target::Module)
    }
}

impl ToTokens for HostRefItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            struct_item,
            trait_impl_item,
            fns_impl_item,
            try_fns_impl_item,
            mut_no_ret_fns_impl_item
        } = self;
        tokens.extend(quote! {
            #struct_item
            #trait_impl_item
            #fns_impl_item
            #try_fns_impl_item
            #mut_no_ret_fns_impl_item
        });
    }
}

#[cfg(test)]
mod tests;
