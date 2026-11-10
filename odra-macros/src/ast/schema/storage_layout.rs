use quote::ToTokens;

use crate::ir::{EnumeratedTypedField, ModuleStructIR};

/// Emits `impl odra::schema::SchemaStorageLayout for Xxx`.
///
/// By default the layout is a module composed of the struct fields, each described by its
/// own `SchemaStorageLayout` implementation. A module can override it with the
/// `layout = <expr>` attribute argument (used by named-key storage modules).
///
/// The custom types of the stored values are always collected from the struct fields, so a
/// type that lives only in the storage ends up in the `types` section of the schema.
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

        let types_chain = self.fields.iter().map(|f| {
            let ty = &f.ty;
            quote::quote!(.chain(<#ty as odra::schema::SchemaStorageLayout>::custom_types()))
        });

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaStorageLayout for #module_ident {
                fn storage_kind() -> odra::schema::StorageKind {
                    #kind
                }

                fn custom_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::vec::Vec::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        #(#types_chain)*
                        .collect::<odra::prelude::BTreeSet<Option<odra::schema::casper_contract_schema::CustomType>>>()
                        .into_iter()
                        .collect()
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
            fields: ir.typed_fields().to_vec()
        })
    }
}

#[cfg(test)]
mod tests;
