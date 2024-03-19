use quote::ToTokens;
use syn::{punctuated::Punctuated, Token};

use crate::{ir::{ModuleStructIR, TypeIR}, utils};

pub struct SchemaErrorsItem {
    module_ident: syn::Ident,
    errors: Vec<syn::Type>
}

impl ToTokens for SchemaErrorsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let errors = self.errors.iter().map(|err| {
            quote::quote! {
                <#err as odra::schema::SchemaErrors>::schema_errors()
            }
        }).collect::<Punctuated<proc_macro2::TokenStream, Token![,]>>();

        let item = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #module_ident {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    let vec: odra::prelude::vec::Vec<odra::prelude::Vec<odra::schema::casper_contract_schema::UserError>> = odra::prelude::vec![
                        #errors
                    ];
                    vec.into_iter().flatten().collect()
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&ModuleStructIR> for SchemaErrorsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident(),
            errors: ir.errors()
        })
    }
}

pub struct SchemaErrorItem {
    ty_ident: syn::Ident,
    errors: Vec<syn::Variant>
}

impl ToTokens for SchemaErrorItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ty_ident;
        let errors = enum_variants(&self.errors);

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #ident {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    #errors
                }
            }
        };
        item.to_tokens(tokens);
    }
}

impl TryFrom<&TypeIR> for SchemaErrorItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let item = ir.self_code();
        let variants = utils::syn::extract_unit_variants(item)?;

        if matches!(item.data, syn::Data::Union(_)) || matches!(item.data, syn::Data::Struct(_)) {
            return Err(syn::Error::new_spanned(
                item,
                "An enum expected."
            ));
        }

        Ok(Self {
            ty_ident: item.ident.clone(),
            errors: variants
        })
    }
}

fn enum_variants(variants: &[syn::Variant]) -> proc_macro2::TokenStream {
    utils::syn::transform_variants(variants, |name, discriminant| {
        quote::quote!(odra::schema::error(#name, "", #discriminant),)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn custom_types_works() {
        let ir = test_utils::mock::module_definition();
        let item = SchemaErrorsItem::try_from(&ir).unwrap();
        let expected = quote::quote!(
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for CounterPack {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    odra::prelude::vec![
                        <Erc20Errors as odra::schema::SchemaErrors>::schema_errors(),
                        <MyErrors as odra::schema::SchemaErrors>::schema_errors()
                    ].into_iter().flatten().collect()
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_odra_error_item() {
        let ty = test_utils::mock::custom_enum();
        let item = SchemaErrorItem::try_from(&ty).unwrap();
        let expected = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for MyType {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    odra::prelude::vec![
                        odra::schema::error("A", "Description of A", 10),
                        odra::schema::error("B", "Description of B", 11),
                    ]
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
