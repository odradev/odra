use crate::{
    ir::{EnumeratedTypedField, ModuleStructIR, VariantIR},
    utils
};
use quote::ToTokens;

/// Emits `impl odra::schema::SchemaErrors for Xxx` for a module: the errors of
/// `#[odra::module(errors = ..)]` and of every field type, each once.
pub struct SchemaErrorsItem {
    module_ident: syn::Ident,
    errors: Option<syn::Type>,
    fields: Vec<EnumeratedTypedField>
}

impl ToTokens for SchemaErrorsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;

        let errors = self
            .errors
            .iter()
            .chain(self.fields.iter().map(|f| &f.ty))
            .map(|ty| quote::quote!(.chain(<#ty as odra::schema::SchemaErrors>::schema_errors())))
            .collect::<Vec<_>>();

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #module_ident {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    odra::prelude::BTreeSet::<odra::schema::casper_contract_schema::UserError>::new()
                        .into_iter()
                        #(#errors)*
                        .collect::<odra::prelude::BTreeSet<odra::schema::casper_contract_schema::UserError>>()
                        .into_iter()
                        .collect::<odra::prelude::Vec<_>>()
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
            fields: ir.typed_fields().to_vec()
        })
    }
}

/// Emits `impl odra::schema::SchemaErrors for Xxx` for an `#[odra::odra_error]` enum: one
/// `odra::schema::error(name, description, code)` per variant.
pub struct SchemaErrorItem {
    ty_ident: syn::Ident,
    /// Name, description and code of each error.
    errors: Vec<(String, String, u16)>
}

impl ToTokens for SchemaErrorItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ty_ident;
        let errors = self.errors.iter().map(|(name, description, code)| {
            quote::quote!(odra::schema::error(#name, #description, #code),)
        });

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #ident {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    odra::prelude::vec![#(#errors)*]
                }
            }
        };
        item.to_tokens(tokens);
    }
}

impl SchemaErrorItem {
    /// `variants` are the enum's, see [crate::ir::TypeIR::error_variants].
    pub fn new(ty_ident: syn::Ident, variants: &[VariantIR]) -> Self {
        Self {
            ty_ident,
            errors: variants
                .iter()
                .map(|v| {
                    (
                        v.ident.to_string(),
                        utils::syn::docs_description(&v.docs),
                        v.code
                    )
                })
                .collect()
        }
    }
}

/// Emits an empty `impl odra::schema::SchemaErrors` for a factory, which has no errors of its own.
pub struct FactorySchemaErrorsItem {
    module_ident: syn::Ident,
}

impl TryFrom<&ModuleStructIR> for FactorySchemaErrorsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident(),
        })
    }
}

impl ToTokens for FactorySchemaErrorsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #module_ident {
                fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                    odra::prelude::vec![]
                }
            }
        };

        item.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests;
