use quote::ToTokens;
use syn::Fields;
use crate::{ast::utils::Named, ir::{TypeIR, TypeKind}, utils};

pub struct SchemaCustomTypeItem {
    ty_ident: syn::Ident,
    kind: TypeKind,
}

impl ToTokens for SchemaCustomTypeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ty_ident.to_string();
        let ident = &self.ty_ident;

        let custom_item = match &self.kind {
            TypeKind::UnitEnum { variants } => custom_enum(name, variants),
            TypeKind::Enum { variants } => custom_complex_enum(name, variants),
            TypeKind::Struct { fields } => custom_struct(name, fields),
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
                        let fields = f.named.iter().map(|f| {
                            let name = f.ident.as_ref().unwrap().to_string();
                            let ty = &f.ty;
                            quote::quote!(odra::schema::struct_member::<#ty>(#name))
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
    let variants = utils::syn::transform_variants(variants, |name, _, discriminant, _| {
        quote::quote!(odra::schema::enum_variant(#name, #discriminant),)
    });

    quote::quote!(odra::schema::custom_enum(#name, #variants))
}

fn custom_complex_enum(enum_name: &str, variants: &[syn::Variant]) -> proc_macro2::TokenStream {
    let variants = utils::syn::transform_variants(variants, |name, fields, discriminant, _| match fields {
        Fields::Named(_) => {
            match fields.len() {
                0 => quote::quote!(odra::schema::enum_variant(#name, #discriminant),),
                _ => {
                    let ty_name = format!("{}::{}", enum_name, name);
                    quote::quote!(odra::schema::enum_custom_type_variant(#name, #discriminant, #ty_name),)
                }
            }
        }
        Fields::Unnamed(_) => {
            match fields.len() {
                0 => quote::quote!(odra::schema::enum_variant(#name, #discriminant),),
                1 => {
                    let ty = fields.iter().next().unwrap().ty.clone();
                    let ty = quote::quote!(#ty);
                    quote::quote!(odra::schema::enum_typed_variant::<#ty>(#name, #discriminant),)
                }
                _ => {
                    let mut ty = proc_macro2::TokenStream::new();
                    syn::token::Paren::default().surround(&mut ty, |tokens| {
                        fields.iter().for_each(|f| {
                            let ty = &f.ty;
                            tokens.extend(quote::quote!(#ty,))
                        });
                    });
                    quote::quote!(odra::schema::enum_typed_variant::<#ty>(#name, #discriminant),)
                }
            }
        }
        Fields::Unit => quote::quote!(odra::schema::enum_variant(#name, #discriminant),),
    });
    quote::quote!(odra::schema::custom_enum(#enum_name, #variants))
}
fn custom_struct(name: &str, fields: &[(syn::Ident, syn::Type)]) -> proc_macro2::TokenStream {
    let members = fields
        .iter()
        .map(|(ident, ty)| {
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
            kind: ir.kind()?,
        })
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
                        .chain(odra::prelude::vec![])
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
    fn test_unit_enum() {
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
                        .chain(odra::prelude::vec![])
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
    fn test_complex_enum() {
        let ir = test_utils::mock::custom_complex_enum();
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
                                    odra::schema::enum_custom_type_variant("A", 0u16, "MyType::A"),
                                    odra::schema::enum_typed_variant::<(u32, String,)>("B", 1u16),
                                    odra::schema::enum_variant("C", 2u16),
                                    odra::schema::enum_variant("D", 3u16),
                                ]
                            ))
                        ])
                        .chain(odra::prelude::vec![
                            Some(odra::schema::custom_struct(
                                "MyType::A", 
                                odra::prelude::vec![
                                    odra::schema::struct_member::<String>("a"),
                                    odra::schema::struct_member::<u32>("b")
                                ]
                            )),
                            Some(odra::schema::custom_struct("MyType::D", odra::prelude::vec![]))
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

    #[test]
    fn test_union() {
        let ir = test_utils::mock::custom_union();
        let item = SchemaCustomTypeItem::try_from(&ir);
        assert!(item.is_err());
    }
}
