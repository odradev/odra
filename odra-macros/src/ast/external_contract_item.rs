use crate::ast::contract_ref_item::RefItem;
use crate::ast::host_ref_item::HostRefItem;
use crate::ast::ref_utils;
use crate::ast::test_parts::{self, TestPartsReexportItem};
use crate::ir::ModuleImplIR;
use derive_try_from_ref::TryFromRef;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// Emits everything `#[odra::external_contract]` adds to a trait: the contract ref and the host
/// ref implementing it, so a contract whose source is not at hand is called like any module.
pub struct ExternalContractImpl {
    /// The trait itself, so it can be used as a bound or implemented by a module.
    trait_item: TokenStream,
    ref_item: RefItem,
    ref_trait_impl: TraitImplItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem
}

impl TryFrom<&ModuleImplIR> for ExternalContractImpl {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        let trait_item = ir.self_code();
        Ok(Self {
            // Only the generated refs may use the trait; do not warn about that.
            trait_item: quote!(#[allow(dead_code)] #trait_item),
            ref_item: ir.try_into()?,
            ref_trait_impl: TraitImplItem::contract_ref(ir)?,
            test_parts: ir.try_into()?,
            test_parts_reexport: ir.try_into()?
        })
    }
}

impl ToTokens for ExternalContractImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            trait_item,
            ref_item,
            ref_trait_impl,
            test_parts,
            test_parts_reexport
        } = self;
        tokens.extend(quote! {
            #trait_item
            #ref_item
            #ref_trait_impl
            #test_parts
            #test_parts_reexport
        });
    }
}

/// `impl Trait for XxxRef { .. }` - makes the ref usable wherever the trait is required.
struct TraitImplItem {
    trait_ident: syn::Ident,
    ref_ident: syn::Ident,
    functions: Vec<syn::ItemFn>
}

impl TraitImplItem {
    fn contract_ref(ir: &ModuleImplIR) -> syn::Result<Self> {
        Ok(Self {
            trait_ident: ir.module_ident()?,
            ref_ident: ir.contract_ref_ident()?,
            functions: ir
                .functions()
                .iter()
                .map(|f| ref_utils::contract_function_item(f, true))
                .collect()
        })
    }

    fn host_ref(ir: &ModuleImplIR) -> syn::Result<Self> {
        Ok(Self {
            trait_ident: ir.module_ident()?,
            ref_ident: ir.host_ref_ident()?,
            functions: ir
                .host_functions()
                .iter()
                .map(|f| ref_utils::host_function_item(f, true))
                .collect()
        })
    }
}

impl ToTokens for TraitImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let trait_ident = &self.trait_ident;
        let ref_ident = &self.ref_ident;
        let functions = &self.functions;
        tokens.extend(quote! {
            impl #trait_ident for #ref_ident {
                #(#functions)*
            }
        })
    }
}

/// The test parts of an external contract: the host ref and `impl Trait for XxxHostRef`; there
/// is no deployer, as there is no code to deploy.
#[derive(TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
struct TestPartsItem {
    #[expr(input.test_parts_mod_ident()?)]
    mod_ident: syn::Ident,
    host_ref: HostRefItem,
    #[expr(TraitImplItem::host_ref(input)?)]
    host_trait_impl: TraitImplItem
}

impl ToTokens for TestPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let host_ref = &self.host_ref;
        let host_trait_impl = &self.host_trait_impl;
        tokens.extend(test_parts::not_wasm32_mod(
            &self.mod_ident,
            quote!(#host_ref #host_trait_impl)
        ));
    }
}

#[cfg(test)]
mod tests;
