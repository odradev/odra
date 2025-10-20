use derive_try_from_ref::TryFromRef;
use quote::ToTokens;

use crate::{
    ast::{
        host_ref_item::{HostRefStructItem, HostRefTraitImplItem},
        ref_utils
    },
    ModuleImplIR
};

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct FactoryHostRefItem {
    struct_item: HostRefStructItem,
    trait_impl_item: HostRefTraitImplItem,
    impl_item: FactoryHostRefImplItem,
    try_impl_item: FactoryHostRefTryImplItem
}

struct FactoryHostRefImplItem {
    ref_ident: syn::Ident,
    factory_fn: syn::ItemFn
}

impl TryFrom<&'_ ModuleImplIR> for FactoryHostRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let fun = module.factory_fn();
        Ok(Self {
            ref_ident: module.host_ref_ident()?,
            factory_fn: ref_utils::host_function_item(&fun, false)
        })
    }
}

impl ToTokens for FactoryHostRefImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ref_ident = &self.ref_ident;
        let factory_fn = &self.factory_fn;
        tokens.extend(quote::quote! {
            impl #ref_ident {
                #factory_fn
            }
        });
    }
}

struct FactoryHostRefTryImplItem {
    ref_ident: syn::Ident,
    factory_fn: syn::ItemFn
}

impl TryFrom<&'_ ModuleImplIR> for FactoryHostRefTryImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let fun = module.factory_fn();

        Ok(Self {
            ref_ident: module.host_ref_ident()?,
            factory_fn: ref_utils::factory_try_function_item(&fun)
        })
    }
}

impl ToTokens for FactoryHostRefTryImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ref_ident = &self.ref_ident;
        let factory_fn = &self.factory_fn;
        tokens.extend(quote::quote! {
            impl #ref_ident {
                #factory_fn
            }
        });
    }
}
