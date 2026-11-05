use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{ref_utils, Target},
    ir::ModuleImplIR,
    utils
};

/// Emits the whole contract ref of a module: the struct, `impl ContractRef`, the impl block with
/// one method per entry point, and the empty schema marker impls.
pub struct RefItem {
    struct_item: ContractRefStructItem,
    trait_impl_item: ContractRefTraitImplItem,
    fns_impl_item: ContractRefFnsImplItem,
    schema_impls_item: SchemaMarkerImplsItem
}

impl RefItem {
    pub(crate) fn try_new(module: &ModuleImplIR, kind: Target) -> syn::Result<Self> {
        let is_trait_impl = module.is_trait_impl();
        let (trait_path, functions) = match kind {
            Target::Module => (
                module.impl_trait_path(),
                module
                    .onchain_functions()
                    .into_iter()
                    .map(|f| ref_utils::contract_function_item(f, is_trait_impl))
                    .collect()
            ),
            Target::Factory => (
                None,
                [
                    module.factory_fn(),
                    module.factory_upgrade_fn(),
                    module.factory_batch_upgrade_fn()
                ]
                .iter()
                .map(|f| ref_utils::factory_contract_function_item(f, false))
                .collect()
            )
        };
        let ref_ident = module.contract_ref_ident();
        Ok(Self {
            struct_item: ContractRefStructItem {
                module_name: module.module_str(),
                ref_ident: ref_ident.clone()
            },
            trait_impl_item: ContractRefTraitImplItem {
                ref_ident: ref_ident.clone()
            },
            fns_impl_item: ContractRefFnsImplItem {
                trait_path,
                ref_ident: ref_ident.clone(),
                functions
            },
            schema_impls_item: SchemaMarkerImplsItem { ref_ident }
        })
    }
}

impl TryFrom<&ModuleImplIR> for RefItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, Target::Module)
    }
}

impl ToTokens for RefItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            struct_item,
            trait_impl_item,
            fns_impl_item,
            schema_impls_item
        } = self;
        tokens.extend(quote! {
            #struct_item
            #trait_impl_item
            #fns_impl_item
            #schema_impls_item
        });
    }
}

/// Emits `pub struct XxxContractRef { env, address, attached_value }`, the handle a contract
/// holds on another contract.
struct ContractRefStructItem {
    module_name: String,
    ref_ident: Ident
}

impl ToTokens for ContractRefStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let doc = format!(" [{}] Contract Ref.", self.module_name);
        let ty_rc_contract_env = utils::ty::rc_contract_env();
        let ty_address = utils::ty::address();
        let ty_u512 = utils::ty::u512();

        tokens.extend(quote! {
            #[doc = #doc]
            pub struct #ref_ident {
                env: #ty_rc_contract_env,
                address: #ty_address,
                attached_value: #ty_u512,
            }
        });
    }
}

/// Emits `impl odra::ContractRef for XxxContractRef`: `new`, `address` and `with_tokens`.
struct ContractRefTraitImplItem {
    ref_ident: Ident
}

impl ToTokens for ContractRefTraitImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let ty_contract_ref = utils::ty::contract_ref();
        let ty_rc_contract_env = utils::ty::rc_contract_env();
        let ty_address = utils::ty::address();
        let ty_u512 = utils::ty::u512();
        let with_tokens_fn = WithTokensFnItem;

        tokens.extend(quote! {
            impl #ty_contract_ref for #ref_ident {
                fn new(env: #ty_rc_contract_env, address: #ty_address) -> Self {
                    Self {
                        env,
                        address,
                        attached_value: #ty_u512::zero()
                    }
                }

                fn address(&self) -> &#ty_address {
                    &self.address
                }

                #with_tokens_fn
            }
        });
    }
}

/// Emits `fn with_tokens(&self, tokens) -> Self`, a copy of the ref that attaches `tokens` to
/// its next call. The host ref and the contract ref have the same three fields, so both use it.
pub(crate) struct WithTokensFnItem;

impl ToTokens for WithTokensFnItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty_u512 = utils::ty::u512();
        let address = utils::ident::address();
        let env = utils::ident::env();
        let attached_value = utils::ident::attached_value();

        tokens.extend(quote! {
            fn with_tokens(&self, tokens: #ty_u512) -> Self {
                Self {
                    #address: self.#address,
                    #env: self.#env.clone(),
                    #attached_value: tokens
                }
            }
        });
    }
}

/// Emits `impl XxxContractRef { .. }` (or `impl Trait for XxxContractRef { .. }`) with one
/// method per entry point, each a `call_contract` with the arguments as runtime args.
struct ContractRefFnsImplItem {
    trait_path: Option<syn::Path>,
    ref_ident: Ident,
    functions: Vec<syn::ItemFn>
}

impl ToTokens for ContractRefFnsImplItem {
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

/// Emits empty `impl SchemaErrors` and `impl SchemaEvents` for the ref outside wasm: a ref
/// contributes nothing to the schema of the contract that holds it.
struct SchemaMarkerImplsItem {
    ref_ident: Ident
}

impl ToTokens for SchemaMarkerImplsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        tokens.extend(quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #ref_ident {}

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for #ref_ident {}
        });
    }
}

#[cfg(test)]
mod tests;
