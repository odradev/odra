use crate::{
    ast::utils::Named,
    ir::{TypeIR, TypeKind},
    utils
};
use quote::ToTokens;
use syn::Fields;

/// Emits the schema impls of an `#[odra::odra_type]`: `SchemaCustomTypes` (the type itself as a
/// custom struct or enum, the structs of its named-field variants, and the custom types of its
/// fields), `NamedCLTyped` (its name) and the `SchemaCustomElement` marker.
pub struct SchemaCustomTypeItem {
    ty_ident: syn::Ident,
    kind: TypeKind
}

impl ToTokens for SchemaCustomTypeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ty_ident.to_string();
        let ident = &self.ty_ident;

        let custom_item = match &self.kind {
            TypeKind::UnitEnum { variants } => custom_enum(name, variants),
            TypeKind::Enum { variants } => custom_complex_enum(name, variants),
            TypeKind::Struct { fields } => custom_struct(name, fields)
        };

        let sub_types = match &self.kind {
            TypeKind::Struct { fields } => fields
                .iter()
                .map(|(_, ty)| {
                    quote::quote!(.chain(<#ty as odra::schema::SchemaCustomTypes>::schema_types()))
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };

        let enum_sub_types = match &self.kind {
            TypeKind::Enum { variants } => variants.iter().filter_map(|v| {
                match &v.fields {
                    Fields::Named(f) => {
                        let fields = f.named.iter().filter_map(|f| {
                            let name = f.ident.as_ref()?.to_string();
                            let ty = &f.ty;
                            Some(quote::quote!(odra::schema::struct_member::<#ty>(#name)))
                        }).collect::<Vec<_>>();
                        let ty_name = format!("{}::{}", name, v.ident);
                        Some(quote::quote!(odra::schema::custom_struct(#ty_name, odra::prelude::vec![#(#fields),*])))
                    }
                    Fields::Unnamed(_) => None,
                    Fields::Unit => None
                }
            }).collect::<Vec<_>>(),
            _ => Vec::new()
        };

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for #ident {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![Some(#custom_item)])
                        .chain(odra::prelude::vec![#(Some(#enum_sub_types)),*])
                        #(#sub_types)*
                        .collect::<odra::prelude::Vec<_>>()
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for #ident {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(odra::prelude::String::from(#name))
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomElement for #ident {}
        };

        item.to_tokens(tokens);
    }
}

fn custom_enum(name: &str, variants: &[syn::Variant]) -> proc_macro2::TokenStream {
    let variants = utils::syn::transform_variants(
        variants,
        |name, _, discriminant, _| quote::quote!(odra::schema::enum_variant(#name, #discriminant),)
    );

    quote::quote!(odra::schema::custom_enum(#name, #variants))
}

fn custom_complex_enum(enum_name: &str, variants: &[syn::Variant]) -> proc_macro2::TokenStream {
    let variants = utils::syn::transform_variants(variants, |name, fields, discriminant, _| {
        match fields {
            Fields::Named(_) => match fields.len() {
                0 => quote::quote!(odra::schema::enum_variant(#name, #discriminant),),
                _ => {
                    let ty_name = format!("{}::{}", enum_name, name);
                    quote::quote!(odra::schema::enum_custom_type_variant(#name, #discriminant, #ty_name),)
                }
            },
            // A tuple variant is typed as its one field, or as the tuple of its fields.
            Fields::Unnamed(unnamed) => {
                let tys = unnamed.unnamed.iter().map(|f| &f.ty).collect::<Vec<_>>();
                match tys.as_slice() {
                    [] => quote::quote!(odra::schema::enum_variant(#name, #discriminant),),
                    [ty] => quote::quote!(odra::schema::enum_typed_variant::<#ty>(#name, #discriminant),),
                    tys => quote::quote!(odra::schema::enum_typed_variant::<(#(#tys,)*)>(#name, #discriminant),)
                }
            }
            Fields::Unit => quote::quote!(odra::schema::enum_variant(#name, #discriminant),)
        }
    });
    quote::quote!(odra::schema::custom_enum(#enum_name, #variants))
}
fn custom_struct(name: &str, fields: &[(syn::Ident, syn::Type)]) -> proc_macro2::TokenStream {
    let members = fields.iter().map(|(ident, ty)| {
        let name = ident.to_string();
        quote::quote!(odra::schema::struct_member::<#ty>(#name))
    });

    quote::quote!(odra::schema::custom_struct(#name, odra::prelude::vec![#(#members,)*]))
}

impl TryFrom<&TypeIR> for SchemaCustomTypeItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ty_ident: ir.name()?,
            kind: ir.kind()?
        })
    }
}

#[cfg(test)]
mod tests;
