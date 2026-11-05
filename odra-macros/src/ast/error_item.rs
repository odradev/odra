use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{schema::SchemaErrorItem},
    ir::{TypeIR, VariantIR}
};

/// Emits everything `#[odra::odra_error]` adds to an error enum: the `From` impls
/// ([OdraErrorItem]) and its schema.
pub struct OdraErrorAttrItem {
    item: OdraErrorItem,
    schema: SchemaErrorItem
}

impl TryFrom<&TypeIR> for OdraErrorAttrItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let variants = ir.error_variants()?;
        Ok(Self {
            item: OdraErrorItem::new(ir, variants),
            schema: SchemaErrorItem::new(ir.ident(), variants)
        })
    }
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

impl OdraErrorItem {
    /// `variants` are the enum's, see [TypeIR::error_variants].
    pub fn new(ty: &TypeIR, variants: &[VariantIR]) -> Self {
        Self {
            item: ty.self_code().clone(),
            ident: ty.ident(),
            variants: variants.iter().map(|v| v.ident.clone()).collect()
        }
    }
}

impl ToTokens for OdraErrorItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            item,
            ident,
            variants
        } = self;

        tokens.extend(quote! {
            #item

            #[cfg(target_arch = "wasm32")]
            #[automatically_derived]
            impl ::core::convert::From<#ident> for OdraError {
                fn from(error: #ident) -> Self {
                    OdraError::user(error as u16)
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            #[automatically_derived]
            impl ::core::convert::From<#ident> for OdraError {
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
