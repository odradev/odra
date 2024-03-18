use quote::ToTokens;
use syn::spanned::Spanned;

use crate::ir::TypeIR;

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
            true => {
                let variants = enum_variants(&self.variants);
                quote::quote! {
                    odra::schema::custom_enum(
                        #name,
                        #variants
                    )
                }
            }
            false => {
                let members = struct_members(&self.fields);
                quote::quote! {
                    odra::schema::custom_struct(
                        #name,
                        #members
                    )
                }
            }
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

fn enum_variants(variants: &[syn::Variant]) -> proc_macro2::TokenStream {
    let mut discriminant = 0u8;
    let variants = variants.iter().map(|v| {
        let name = v.ident.to_string();
        if let Some((_, syn::Expr::Lit(lit))) = &v.discriminant {
            if let syn::Lit::Int(int) = &lit.lit {
                discriminant = int.base10_parse().unwrap();
            }
        };
        let result = quote::quote! {
            odra::schema::enum_variant(#name, #discriminant),
        };
        discriminant += 1;
        result
    });

    quote::quote! {
        vec![
            #(#variants)*
        ]
    }
}

fn struct_members(fields: &[syn::Field]) -> proc_macro2::TokenStream {
    let members = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        quote::quote! {
            odra::schema::struct_member::<#ty>(#name),
        }
    });

    quote::quote! {
        vec![
            #(#members)*
        ]
    }
}

impl TryFrom<&TypeIR> for SchemaCustomTypeItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let item = ir.self_code();
        let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &item.data {
            fields
                .iter()
                .map(|f| {
                    if f.ident.is_none() {
                        Err(syn::Error::new(f.span(), "Unnamed field"))
                    } else {
                        Ok(f.clone())
                    }
                })
                .collect()
        } else {
            Ok(vec![])
        }?;

        let variants = if let syn::Data::Enum(syn::DataEnum { variants, .. }) = &item.data {
            let is_valid = variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit));
            if is_valid {
                Ok(variants.into_iter().cloned().collect())
            } else {
                Err(syn::Error::new_spanned(
                    variants,
                    "Expected a unit enum variant."
                ))
            }
        } else {
            Ok(vec![])
        }?;

        if matches!(item.data, syn::Data::Union(_)) {
            return Err(syn::Error::new_spanned(
                item,
                "Struct with named fields expected"
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
