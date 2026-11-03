use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{contract_ref_item::WithTokensFnItem, ref_utils},
    ir::ModuleImplIR,
    utils
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
            module_name: module.module_str()?,
            ref_ident: module.host_ref_ident()?
        })
    }
}

impl ToTokens for HostRefStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let doc = format!(" [{}] Host Ref.", self.module_name);
        let address = utils::ident::address();
        let env = utils::ident::env();
        let attached_value = utils::ident::attached_value();
        let ty_address = utils::ty::address();
        let ty_host_env = utils::ty::host_env();
        let ty_u512 = utils::ty::u512();

        tokens.extend(quote! {
            #[doc = #doc]
            pub struct #ref_ident {
                #address: #ty_address,
                #env: #ty_host_env,
                #attached_value: #ty_u512
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
            ref_ident: module.host_ref_ident()?,
            register: matches!(module, ModuleImplIR::Impl(_))
        })
    }
}

impl ToTokens for HostRefTraitImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let ty_host_ref = utils::ty::host_ref();
        let ty_address = utils::ty::address();
        let ty_host_env = utils::ty::host_env();
        let ty_has_ident = utils::ty::has_ident();
        let ty_caller_provider = utils::ty::entry_point_caller_provider();
        let ty_event_instance = utils::ty::event_instance();
        let ty_from_bytes = utils::ty::from_bytes();
        let ty_event_error = utils::ty::event_error();
        let ty_call_result = utils::ty::contract_call_result();
        let address = utils::ident::address();
        let env = utils::ident::env();
        let attached_value = utils::ident::attached_value();
        let contract_address = utils::ident::contract_address();
        let with_tokens_fn = WithTokensFnItem;

        let register = self.register.then(|| {
            quote! {
                #env.register_contract_ref(
                    #address,
                    <Self as #ty_has_ident>::ident(),
                    <Self as #ty_caller_provider>::entry_points_caller
                );
            }
        });

        tokens.extend(quote! {
            impl #ty_host_ref for #ref_ident {
                fn new(#address: #ty_address, #env: #ty_host_env) -> Self {
                    #register
                    Self {
                        #address,
                        #env,
                        #attached_value: Default::default()
                    }
                }

                #with_tokens_fn

                fn #contract_address(&self) -> #ty_address {
                    self.#address
                }

                fn #env(&self) -> &#ty_host_env {
                    &self.#env
                }

                fn get_event<T>(&self, index: i32) -> Result<T, #ty_event_error>
                where
                    T: #ty_from_bytes + #ty_event_instance
                {
                    self.#env.get_event(self, index)
                }

                fn last_call(&self) -> #ty_call_result {
                    self.#env.last_call_result(self.#address)
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
            module_ident: module.module_ident()?,
            ref_ident: module.host_ref_ident()?
        })
    }
}

impl ToTokens for HasIdentTraitImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let ref_ident = &self.ref_ident;
        let ty_has_ident = utils::ty::has_ident();
        let ty_string = utils::ty::string();
        let ident = utils::ident::ident();

        tokens.extend(quote! {
            impl #ty_has_ident for #ref_ident {
                fn #ident() -> #ty_string {
                    #module_ident::#ident()
                }
                fn contract_name() -> #ty_string {
                    <#module_ident as #ty_has_ident>::contract_name()
                }
            }
        });
    }
}

/// Emits one `impl XxxHostRef { .. }` (or `impl Trait for XxxHostRef { .. }`) holding the given
/// functions. The host ref has three: the entry points as written, their `try_` versions
/// returning `OdraResult`, and for a mutating entry point with a result, a `_no_ret` version.
struct HostRefFnsImplItem {
    trait_ident: Option<Ident>,
    ref_ident: Ident,
    functions: Vec<syn::ItemFn>
}

impl ToTokens for HostRefFnsImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let functions = &self.functions;
        let for_trait = self.trait_ident.as_ref().map(|t| quote!(#t for));

        tokens.extend(quote! {
            impl #for_trait #ref_ident {
                #(#functions)*
            }
        });
    }
}

/// Emits the whole host ref of a module: the struct, `impl HostRef`, and the impl blocks with
/// the entry points, their `try_` versions and the `_no_ret` versions.
pub struct HostRefItem {
    struct_item: HostRefStructItem,
    trait_impl_item: HostRefTraitImplItem,
    fns_impl_item: HostRefFnsImplItem,
    try_fns_impl_item: HostRefFnsImplItem,
    mut_no_ret_fns_impl_item: Option<HostRefFnsImplItem>
}

impl TryFrom<&ModuleImplIR> for HostRefItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let ref_ident = module.host_ref_ident()?;
        let is_trait_impl = module.is_trait_impl();
        let host_functions = module.host_functions()?;
        let mut_ret_functions = module.host_mut_ret_functions()?;

        let fns_impl_item = HostRefFnsImplItem {
            trait_ident: module.impl_trait_ident(),
            ref_ident: ref_ident.clone(),
            functions: host_functions
                .iter()
                .map(|f| ref_utils::host_function_item(f, is_trait_impl))
                .collect()
        };
        let try_fns_impl_item = HostRefFnsImplItem {
            trait_ident: None,
            ref_ident: ref_ident.clone(),
            functions: host_functions
                .iter()
                .map(ref_utils::host_try_function_item)
                .collect()
        };
        let mut_no_ret_fns_impl_item = (!mut_ret_functions.is_empty()).then(|| {
            HostRefFnsImplItem {
                trait_ident: None,
                ref_ident,
                functions: mut_ret_functions
                    .iter()
                    .flat_map(|f| {
                        [
                            ref_utils::host_mut_no_ret_function_item(f),
                            ref_utils::host_mut_try_no_ret_function_item(f)
                        ]
                    })
                    .collect()
            }
        });

        Ok(Self {
            struct_item: HostRefStructItem::try_from(module)?,
            trait_impl_item: HostRefTraitImplItem::try_from(module)?,
            fns_impl_item,
            try_fns_impl_item,
            mut_no_ret_fns_impl_item
        })
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
