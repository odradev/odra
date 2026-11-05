use crate::ir::errors;
use crate::utils;
use proc_macro2::TokenStream;

/// Emits `impl XxxContractRef { .. }` and `impl XxxHostRef { .. }` with the items of an
/// `#[odra::ref_helpers] impl Xxx { .. }` block.
///
/// The impl block is not emitted for `Xxx` itself; its items are copied verbatim, so a helper
/// can call any function both refs have - the entry points and `contract_address()`. Each impl is
/// emitted in an anonymous const block that brings the trait providing `contract_address()` into
/// scope (`ContractAddress` for the contract ref, `HostRef` for the host ref), so the helper body
/// can call it without the user importing anything and without changing what is in scope outside.
pub struct RefHelpersItem {
    contract_ref_impl: syn::ItemImpl,
    host_ref_impl: syn::ItemImpl
}

impl TryFrom<&'_ TokenStream> for RefHelpersItem {
    type Error = syn::Error;

    fn try_from(item: &'_ TokenStream) -> Result<Self, Self::Error> {
        let item_impl = match syn::parse2::<syn::Item>(item.clone())? {
            syn::Item::Impl(item_impl) => item_impl,
            other => return Err(errors::ref_helpers_on_non_impl(&other))
        };
        if let Some((_, path, _)) = &item_impl.trait_ {
            return Err(errors::ref_helpers_on_trait_impl(path));
        }
        if !item_impl.generics.params.is_empty() {
            return Err(errors::ref_helpers_with_generics(&item_impl.generics));
        }
        let path = match item_impl.self_ty.as_ref() {
            syn::Type::Path(syn::TypePath { qself: None, path }) => path,
            ty => return Err(errors::ref_helpers_on_non_path(ty))
        };
        // A parsed path has at least one segment.
        let Some(last) = path.segments.last() else {
            return Err(errors::ref_helpers_on_non_path(&item_impl.self_ty));
        };
        if !last.arguments.is_none() {
            return Err(errors::ref_helpers_with_generic_args(&last.arguments));
        }

        let with_suffix = |suffix: &str| -> syn::Type {
            let mut path = path.clone();
            if let Some(last) = path.segments.last_mut() {
                last.ident =
                    syn::Ident::new(&format!("{}{}", last.ident, suffix), last.ident.span());
            }
            syn::Type::Path(syn::TypePath { qself: None, path })
        };
        let contract_ref_impl = syn::ItemImpl {
            self_ty: Box::new(with_suffix("ContractRef")),
            ..item_impl.clone()
        };
        let host_ref_impl = syn::ItemImpl {
            self_ty: Box::new(with_suffix("HostRef")),
            ..item_impl
        };
        Ok(Self {
            contract_ref_impl,
            host_ref_impl
        })
    }
}

impl quote::ToTokens for RefHelpersItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let contract_ref_impl = &self.contract_ref_impl;
        let host_ref_impl = &self.host_ref_impl;
        let not_wasm32 = utils::attr::not_wasm32();
        tokens.extend(quote::quote! {
            const _: () = {
                use odra::ContractAddress as _;
                #contract_ref_impl
            };

            #not_wasm32
            const _: () = {
                use odra::host::HostRef as _;
                #host_ref_impl
            };
        });
    }
}

#[cfg(test)]
mod tests;
