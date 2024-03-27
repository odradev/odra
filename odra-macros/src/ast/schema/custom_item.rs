use quote::ToTokens;
use crate::{ast::utils::Named, ir::TypeIR, utils};

pub struct SchemaCustomTypeItem {
    ty_ident: syn::Ident,
    is_enum: bool,
    fields: Vec<syn::Field>,
    variants: Vec<syn::Variant>
}

impl ToTokens for SchemaCustomTypeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ty_ident.to_string();
        let ident = &self.ty_ident;

        let custom_item = match self.is_enum {
            true => custom_enum(name, &self.variants),
            false => custom_struct(name, &self.fields)
        };

        let sub_types = self.fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote::quote!(.chain(<#ty as odra::schema::SchemaCustomTypes>::schema_types()))
            })
            .collect::<Vec<_>>();

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for #ident {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![Some(#custom_item)])
                        #(#sub_types)*
                        .collect::<Vec<_>>()
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for #ident {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(String::from(#name))
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
    let variants = utils::syn::transform_variants(variants, |name, discriminant| {
        quote::quote!(odra::schema::enum_variant(#name, #discriminant),)
    });

    quote::quote!(odra::schema::custom_enum(#name, #variants))
}

fn custom_struct(name: &str, fields: &[syn::Field]) -> proc_macro2::TokenStream {
    let members = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        quote::quote! {
            odra::schema::struct_member::<#ty>(#name),
        }
    });

    quote::quote!(odra::schema::custom_struct(#name, odra::prelude::vec![#(#members)*]))
}

impl TryFrom<&TypeIR> for SchemaCustomTypeItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let item = ir.self_code();
        if matches!(item, syn::Item::Struct(_) | syn::Item::Enum(_)) {
            let fields = if let syn::Item::Struct(s) = item {
                utils::syn::extract_named_field(s)?
            } else {
                vec![]
            };

            let variants = if let syn::Item::Enum(e) = item {
                utils::syn::extract_unit_variants(e)?
            } else {
                vec![]
            };

            Ok(Self {
                ty_ident: ir.name()?,
                is_enum: ir.is_enum(),
                fields,
                variants
            })
        } else {
            Err(syn::Error::new_spanned(
                item,
                "Struct with named fields or a unit variants enum expected"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn test_struct() {
        let ir = test_utils::mock::custom_struct();
        let item = SchemaCustomTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for MyType {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![
                            Some(odra::schema::custom_struct(
                                "MyType",
                                odra::prelude::vec![
                                    odra::schema::struct_member::<String>("a"),
                                    odra::schema::struct_member::<u32>("b"),
                                ]
                            ))
                        ])
                        .chain(<String as odra::schema::SchemaCustomTypes>::schema_types())
                        .chain(<u32 as odra::schema::SchemaCustomTypes>::schema_types())
                        .collect::<Vec<_>>()
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for MyType {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(String::from(
                        "MyType"
                    ))
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomElement for MyType {}
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_enum() {
        let ir = test_utils::mock::custom_enum();
        let item = SchemaCustomTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for MyType {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![
                            Some(odra::schema::custom_enum(
                                "MyType",
                                odra::prelude::vec![
                                    odra::schema::enum_variant("A", 10u16),
                                    odra::schema::enum_variant("B", 11u16),
                                ]
                            ))
                        ])
                        .collect::<Vec<_>>()
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for MyType {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(String::from(
                        "MyType"
                    ))
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomElement for MyType {}
        );

        test_utils::assert_eq(item, expected);
    }
}
