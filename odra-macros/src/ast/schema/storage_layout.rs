use quote::ToTokens;

use crate::ir::{EnumeratedTypedField, ModuleStructIR};

/// Generates the `odra::schema::SchemaStorageLayout` implementation of a module.
///
/// By default the layout is a module composed of the struct fields, each described by its
/// own `SchemaStorageLayout` implementation. A module can override it with the
/// `layout = <expr>` attribute argument (used by named-key storage modules).
pub struct SchemaStorageLayoutItem {
    module_ident: syn::Ident,
    layout: Option<syn::Expr>,
    fields: Vec<EnumeratedTypedField>
}

impl ToTokens for SchemaStorageLayoutItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;

        let kind = match &self.layout {
            Some(layout) => quote::quote!(#layout),
            None => {
                let fields = self.fields.iter().map(|f| {
                    let name = f.ident.to_string();
                    let idx = f.idx;
                    let ty = &f.ty;
                    quote::quote!(odra::schema::StorageField::new(
                        #name,
                        #idx,
                        <#ty as odra::schema::SchemaStorageLayout>::storage_kind()
                    ))
                });
                quote::quote!(odra::schema::StorageKind::Module {
                    fields: odra::prelude::vec![#(#fields),*]
                })
            }
        };

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaStorageLayout for #module_ident {
                fn storage_kind() -> odra::schema::StorageKind {
                    #kind
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&ModuleStructIR> for SchemaStorageLayoutItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident(),
            layout: ir.storage_layout(),
            fields: ir.typed_fields()?
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn storage_layout_works() {
        let ir = test_utils::mock::module_definition();
        let item = SchemaStorageLayoutItem::try_from(&ir).unwrap();
        let expected = quote::quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaStorageLayout for CounterPack {
                fn storage_kind() -> odra::schema::StorageKind {
                    odra::schema::StorageKind::Module {
                        fields: odra::prelude::vec![
                            odra::schema::StorageField::new(
                                "counter0",
                                1u8,
                                <SubModule<Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                            ),
                            odra::schema::StorageField::new(
                                "counter1",
                                2u8,
                                <SubModule<Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                            ),
                            odra::schema::StorageField::new(
                                "counter2",
                                3u8,
                                <SubModule<Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                            ),
                            odra::schema::StorageField::new(
                                "counters",
                                4u8,
                                <Var<u32> as odra::schema::SchemaStorageLayout>::storage_kind()
                            ),
                            odra::schema::StorageField::new(
                                "counters_map",
                                5u8,
                                <Mapping<u8, Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                            )
                        ]
                    }
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn explicit_layout_works() {
        let module = quote::quote!(
            pub struct Decimals;
        );
        let attr = quote::quote!(layout = odra::schema::StorageKind::named_key::<u8>("decimals"));
        let ir = ModuleStructIR::try_from((&attr, &module)).unwrap();
        let item = SchemaStorageLayoutItem::try_from(&ir).unwrap();
        let expected = quote::quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaStorageLayout for Decimals {
                fn storage_kind() -> odra::schema::StorageKind {
                    odra::schema::StorageKind::named_key::<u8>("decimals")
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }
}
