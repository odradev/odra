use derive_try_from_ref::TryFromRef;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    ast::{
        host_ref_item::{HostRefStructItem, HostRefTraitImplItem},
        ref_utils
    },
    ModuleImplIR
};

/// Emits the host ref of a factory: the same struct and HostRef impl as a module's, with the
/// three factory functions and their try_ versions as its methods.
#[derive(TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct FactoryHostRefItem {
    struct_item: HostRefStructItem,
    trait_impl_item: HostRefTraitImplItem,
    impl_item: FactoryHostRefImplItem,
    try_impl_item: FactoryHostRefTryImplItem
}

impl ToTokens for FactoryHostRefItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            struct_item,
            trait_impl_item,
            impl_item,
            try_impl_item
        } = self;
        tokens.extend(quote! {
            #struct_item
            #trait_impl_item
            #impl_item
            #try_impl_item
        });
    }
}

struct FactoryHostRefImplItem {
    ref_ident: syn::Ident,
    factory_fns: Vec<syn::ItemFn>
}

impl TryFrom<&ModuleImplIR> for FactoryHostRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let fns = [module.factory_fn(), module.factory_upgrade_fn(), module.factory_batch_upgrade_fn()];
        Ok(Self {
            ref_ident: module.host_ref_ident()?,
            factory_fns: fns.iter().map(|f| ref_utils::host_function_item(f, false)).collect()
        })
    }
}

impl ToTokens for FactoryHostRefImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let factory_fns = &self.factory_fns;
        tokens.extend(quote! {
            impl #ref_ident {
                #(#factory_fns)*
            }
        });
    }
}

struct FactoryHostRefTryImplItem {
    ref_ident: syn::Ident,
    factory_fns: Vec<syn::ItemFn>
}

impl TryFrom<&ModuleImplIR> for FactoryHostRefTryImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let fns = [module.factory_fn(), module.factory_upgrade_fn(), module.factory_batch_upgrade_fn()];

        Ok(Self {
            ref_ident: module.host_ref_ident()?,
            factory_fns: fns.iter().map(ref_utils::factory_try_function_item).collect()
        })
    }
}

impl ToTokens for FactoryHostRefTryImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ref_ident = &self.ref_ident;
        let factory_fns = &self.factory_fns;
        tokens.extend(quote! {
            impl #ref_ident {
                #(#factory_fns)*
            }
        });
    }
}
