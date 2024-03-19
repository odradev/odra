use quote::ToTokens;
use crate::{ir::TypeIR, utils};

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

        let item = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomType for #ident {
                fn custom_ty() -> Option<odra::schema::casper_contract_schema::CustomType> {
                    Some(#custom_item)
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for #ident {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(String::from(#name))
                }
            }
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

    quote::quote!(odra::schema::custom_struct(#name, vec![#(#members)*]))
}

impl TryFrom<&TypeIR> for SchemaCustomTypeItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let item = ir.self_code();
        let fields = utils::syn::extract_named_field(item)?;
        let variants = utils::syn::extract_unit_variants(item)?;

        if matches!(item.data, syn::Data::Union(_)) {
            return Err(syn::Error::new_spanned(
                item,
                "Struct with named fields or a unit variants enum expected"
            ));
        }

        Ok(Self {
            ty_ident: item.ident.clone(),
            is_enum: ir.is_enum(),
            fields,
            variants
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
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomType for MyType {
                fn custom_ty() -> Option<odra::schema::casper_contract_schema::CustomType> {
                    Some(odra::schema::custom_struct(
                        "MyType",
                        vec![
                            odra::schema::struct_member::<String>("a"),
                            odra::schema::struct_member::<u32>("b"),
                        ]
                    ))
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for MyType {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(String::from(
                        "MyType"
                    ))
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_enum() {
        let ir = test_utils::mock::custom_enum();
        let item = SchemaCustomTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomType for MyType {
                fn custom_ty() -> Option<odra::schema::casper_contract_schema::CustomType> {
                    Some(odra::schema::custom_enum(
                        "MyType",
                        vec![
                            odra::schema::enum_variant("A", 10u8),
                            odra::schema::enum_variant("B", 11u8),
                        ]
                    ))
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::NamedCLTyped for MyType {
                fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                    odra::schema::casper_contract_schema::NamedCLType::Custom(String::from(
                        "MyType"
                    ))
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }
}
