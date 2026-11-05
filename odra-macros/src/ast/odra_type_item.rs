use derive_try_from_ref::TryFromRef;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::{
    ast::{events_item::HasEventsImplItem, schema::SchemaCustomTypeItem},
    ir::{TypeIR, TypeKind, VariantIR},
    utils
};

/// Emits everything `#[odra::odra_type]` adds to a type: the derives and trait impls
/// ([OdraTypeItem]) and its schema.
#[derive(TryFromRef)]
#[source(TypeIR)]
#[err(syn::Error)]
pub struct OdraTypeAttrItem {
    item: OdraTypeItem,
    schema: SchemaCustomTypeItem
}

impl ToTokens for OdraTypeAttrItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);
        self.schema.to_tokens(tokens);
    }
}

/// Emits the user's struct or enum with `#[derive(Clone, PartialEq, Eq, Debug)]`, then
/// `impl FromBytes`, `impl ToBytes`, `impl CLTyped` and an empty `impl HasEvents` for it.
///
/// A unit enum is one byte, its discriminant. Any other enum is one byte of variant index
/// followed by the variant's fields in order, and a struct is its fields in order.
pub struct OdraTypeItem {
    item: syn::Item,
    ident: Ident,
    kind: TypeKind,
    has_events_impl: HasEventsImplItem
}

impl TryFrom<&TypeIR> for OdraTypeItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            item: ir.self_code().clone(),
            ident: ir.ident(),
            kind: ir.kind().clone(),
            has_events_impl: ir.try_into()?
        })
    }
}

impl ToTokens for OdraTypeItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item = &self.item;
        let derive_attr = utils::attr::common_derive_attr();
        let from_bytes_impl = self.impl_from_bytes();
        let to_bytes_impl = self.impl_to_bytes();
        let cl_typed_impl = self.impl_cl_typed();
        let has_events_impl = &self.has_events_impl;

        tokens.extend(quote! {
            #derive_attr
            #item
            #from_bytes_impl
            #to_bytes_impl
            #cl_typed_impl
            #has_events_impl
        });
    }
}

impl OdraTypeItem {
    /// `impl FromBytes`: `fn from_bytes(bytes) -> Result<(Self, rest), Error>`.
    fn impl_from_bytes(&self) -> TokenStream {
        let ident = &self.ident;
        let ty_from_bytes = utils::ty::from_bytes();
        let ty_error = utils::ty::bytes_err();
        let bytes = utils::ident::bytes();
        let read = utils::ident::result();

        let body = match &self.kind {
            TypeKind::Struct { fields } => {
                let names = fields.iter().map(|(name, _)| name).collect::<Vec<_>>();
                quote! {
                    #(let (#names, #bytes) = #ty_from_bytes::from_bytes(#bytes)?;)*
                    Ok((Self { #(#names),* }, #bytes))
                }
            }
            TypeKind::UnitEnum { variants } => {
                let arms = variants.iter().map(|v| {
                    let value = variant_constructor(v);
                    quote!(x if x == #value as u8 => Ok((#value, #bytes)))
                });
                quote! {
                    let (#read, #bytes): (u8, _) = #ty_from_bytes::from_bytes(#bytes)?;
                    match #read {
                        #(#arms,)*
                        _ => Err(#ty_error::Formatting),
                    }
                }
            }
            TypeKind::Enum { variants } => {
                let arms = variants.iter().enumerate().map(|(index, v)| {
                    let index = index as u8;
                    let names = variant_field_names(v);
                    let value = variant_constructor(v);
                    quote! {
                        #index => {
                            #(let (#names, #bytes) = #ty_from_bytes::from_bytes(#bytes)?;)*
                            Ok((#value, #bytes))
                        }
                    }
                });
                quote! {
                    let (#read, #bytes): (u8, _) = #ty_from_bytes::from_bytes(#bytes)?;
                    match #read {
                        #(#arms,)*
                        _ => Err(#ty_error::Formatting),
                    }
                }
            }
        };

        quote! {
            impl #ty_from_bytes for #ident {
                fn from_bytes(#bytes: &[u8]) -> Result<(Self, &[u8]), #ty_error> {
                    #body
                }
            }
        }
    }

    /// `impl ToBytes`: `fn to_bytes(&self)` and `fn serialized_length(&self)`.
    fn impl_to_bytes(&self) -> TokenStream {
        let ident = &self.ident;
        let ty_to_bytes = utils::ty::to_bytes();
        let ty_error = utils::ty::bytes_err();
        let ty_vec = utils::ty::vec();
        let result = utils::ident::result();
        let u8_len = quote!(odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH);

        let (to_bytes_body, serialized_length_body) = match &self.kind {
            TypeKind::Struct { fields } => {
                let names = fields.iter().map(|(name, _)| name).collect::<Vec<_>>();
                (
                    quote! {
                        let mut #result = #ty_vec::with_capacity(self.serialized_length());
                        #(#result.extend(#ty_to_bytes::to_bytes(&self.#names)?);)*
                        Ok(#result)
                    },
                    quote! {
                        let mut #result = 0;
                        #(#result += self.#names.serialized_length();)*
                        #result
                    }
                )
            }
            TypeKind::UnitEnum { .. } => (
                quote!(Ok(odra::prelude::vec![self.clone() as u8])),
                quote!(#u8_len)
            ),
            TypeKind::Enum { variants } => {
                let to_bytes_arms = variants.iter().enumerate().map(|(index, v)| {
                    let index = index as u8;
                    let pattern = variant_pattern(v);
                    let names = variant_field_names(v);
                    quote! {
                        #pattern => {
                            let mut #result = odra::prelude::vec![#index];
                            #(#result.extend_from_slice(&#names.to_bytes()?);)*
                            Ok(#result)
                        }
                    }
                });
                let length_arms = variants.iter().map(|v| {
                    let pattern = variant_pattern(v);
                    let names = variant_field_names(v);
                    quote!(#pattern => #u8_len #(+ #names.serialized_length())*)
                });
                (
                    quote!(match self { #(#to_bytes_arms),* }),
                    quote!(match self { #(#length_arms),* })
                )
            }
        };

        quote! {
            impl #ty_to_bytes for #ident {
                fn to_bytes(&self) -> Result<#ty_vec<u8>, #ty_error> {
                    #to_bytes_body
                }

                fn serialized_length(&self) -> usize {
                    #serialized_length_body
                }
            }
        }
    }

    /// `impl CLTyped`: `U8` for a unit enum, `Any` for everything else.
    fn impl_cl_typed(&self) -> TokenStream {
        let ident = &self.ident;
        let ty_cl_typed = utils::ty::cl_typed();
        let ty_cl_type = utils::ty::cl_type();
        let cl_type = match &self.kind {
            TypeKind::UnitEnum { .. } => quote!(#ty_cl_type::U8),
            TypeKind::Enum { .. } | TypeKind::Struct { .. } => quote!(#ty_cl_type::Any)
        };

        quote! {
            impl #ty_cl_typed for #ident {
                fn cl_type() -> #ty_cl_type {
                    #cl_type
                }
            }
        }
    }
}

/// The names the variant's fields are bound to: their own for named fields, `f0`, `f1`, .. for
/// a tuple variant.
fn variant_field_names(variant: &VariantIR) -> Vec<Ident> {
    variant
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| match &field.ident {
            Some(ident) => ident.clone(),
            None => format_ident!("f{}", index)
        })
        .collect()
}

/// `Self::A { a, b }`, `Self::B(f0, f1)` or `Self::C`, matching the variant with its fields
/// bound to [variant_field_names].
fn variant_pattern(variant: &VariantIR) -> TokenStream {
    let ident = &variant.ident;
    let names = variant_field_names(variant);
    match &variant.fields {
        syn::Fields::Unit => quote!(Self::#ident),
        syn::Fields::Named(_) => quote!(Self::#ident { #(#names),* }),
        syn::Fields::Unnamed(_) => quote!(Self::#ident(#(#names),*))
    }
}

/// The variant built from the bound [variant_field_names]; for a unit variant, the same tokens
/// as its pattern.
fn variant_constructor(variant: &VariantIR) -> TokenStream {
    let ident = &variant.ident;
    let names = variant_field_names(variant);
    match &variant.fields {
        syn::Fields::Unit => quote!(Self::#ident),
        syn::Fields::Named(_) => quote!(Self::#ident { #(#names,)* }),
        syn::Fields::Unnamed(_) => quote!(Self::#ident(#(#names,)*))
    }
}

#[cfg(test)]
mod tests;
