use derive_try_from_ref::TryFromRef;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{schema::SchemaErrorItem},
    ir::TypeIR,
    utils
};

/// Emits everything `#[odra::odra_error]` adds to an error enum: the `From` impls
/// ([OdraErrorItem]) and its schema.
#[derive(TryFromRef)]
#[source(TypeIR)]
#[err(syn::Error)]
pub struct OdraErrorAttrItem {
    item: OdraErrorItem,
    schema: SchemaErrorItem
}

impl ToTokens for OdraErrorAttrItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
        self.schema.to_tokens(tokens);
    }
}

/// Emits the user's enum and `impl From<Xxx> for OdraError` twice: in wasm the error is only
/// its code, outside wasm it carries the variant's name too, for test output.
pub struct OdraErrorItem {
    item: syn::Item,
    ident: Ident,
    variants: Vec<Ident>
}

impl TryFrom<&TypeIR> for OdraErrorItem {
    type Error = syn::Error;

    fn try_from(ty: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            item: ty.self_code().clone(),
            ident: ty.ident(),
            variants: ty.error_variants()?.iter().map(|v| v.ident.clone()).collect()
        })
    }
}

impl ToTokens for OdraErrorItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            item,
            ident,
            variants
        } = self;
        let ty_odra_error = utils::ty::odra_error();
        let ty_from = utils::ty::from(ident);
        let wasm32_attr = utils::attr::wasm32();
        let not_wasm32_attr = utils::attr::not_wasm32();
        let derived_attr = utils::attr::automatically_derived();

        tokens.extend(quote! {
            #item

            #wasm32_attr
            #derived_attr
            impl #ty_from for #ty_odra_error {
                fn from(error: #ident) -> Self {
                    #ty_odra_error::user(error as u16)
                }
            }

            #not_wasm32_attr
            #derived_attr
            impl #ty_from for #ty_odra_error {
                fn from(error: #ident) -> Self {
                    match error {
                        #(#ident::#variants => OdraError::user(error as u16, stringify!(#variants)),)*
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
