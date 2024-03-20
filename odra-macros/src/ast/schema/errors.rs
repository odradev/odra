use quote::ToTokens;
use crate::{ir::{EnumeratedTypedField, ModuleStructIR, TypeIR}, utils};

pub struct SchemaErrorsItem {
    module_ident: syn::Ident,
    errors: Vec<syn::Type>,
    fields: Vec<EnumeratedTypedField>
}

impl ToTokens for SchemaErrorsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;

        let errors = self.errors
            .iter()
            .chain(self.fields.iter().map(|f| &f.ty))
            .map(|ty| {
                quote::quote!(.chain(<#ty as odra::schema::SchemaErrors>::schema_errors()))
            })
            .collect::<Vec<_>>();

        let item = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #module_ident {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    odra::prelude::BTreeSet::<odra::schema::casper_contract_schema::UserError>::new()
                        .into_iter()
                        #(#errors)*
                        .collect::<odra::prelude::BTreeSet<odra::schema::casper_contract_schema::UserError>>()
                        .into_iter()
                        .collect::<Vec<_>>()
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
            errors: ir.errors(),
            fields: ir.typed_fields()?
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
                    odra::prelude::BTreeSet::<odra::schema::casper_contract_schema::UserError>::new()
                        .into_iter()
                        .chain(<Erc20Errors as odra::schema::SchemaErrors>::schema_errors())
                        .chain(<MyErrors as odra::schema::SchemaErrors>::schema_errors())
                        .chain(<SubModule<Counter> as odra::schema::SchemaErrors>::schema_errors())
                        .chain(<SubModule<Counter> as odra::schema::SchemaErrors>::schema_errors())
                        .chain(<SubModule<Counter> as odra::schema::SchemaErrors>::schema_errors())
                        .chain(<Var<u32> as odra::schema::SchemaErrors>::schema_errors())
                        .chain(<Mapping<u8, Counter> as odra::schema::SchemaErrors>::schema_errors())
                        .collect::<odra::prelude::BTreeSet<odra::schema::casper_contract_schema::UserError>>()
                        .into_iter()
                        .collect::<Vec<_>>()
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
                        odra::schema::error("A", "", 10u16),
                        odra::schema::error("B", "", 11u16),
                    ]
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
