use std::collections::HashSet;

use quote::ToTokens;

use crate::{ir::ModuleImplIR, utils::ty};

/// Emits `impl odra::schema::SchemaCustomTypes for Xxx`: the custom types of every argument
/// and return type of the entry points, each type once, plus those of the events.
pub struct SchemaCustomTypesItem {
    module_ident: syn::Ident,
    /// The argument and return types of every function, in order, with repeats.
    types: Vec<syn::Type>
}

impl ToTokens for SchemaCustomTypesItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;

        let mut tmp = HashSet::<String>::new();
        let mut chain = vec![];
        self.types.iter().for_each(|t| {
            let v = quote::quote!(.chain(<#t as odra::schema::SchemaCustomTypes>::schema_types()));
            if tmp.insert(v.to_string()) {
                chain.push(v);
            }
        });

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for #module_ident {
                fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        #(#chain)*
                        .chain(<Self as odra::schema::SchemaEvents>::custom_types())
                        .collect()
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&ModuleImplIR> for SchemaCustomTypesItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident()?,
            types: ir
                .functions()
                .iter()
                .flat_map(|f| {
                    let args = f.typed_args();
                    let mut types = args
                        .iter()
                        .map(|arg| ty::unreferenced_ty(&arg.ty))
                        .collect::<Vec<_>>();
                    if let syn::ReturnType::Type(_, t) = f.return_type() {
                        types.push(*t);
                    };
                    types
                })
                .collect()
        })
    }
}

#[cfg(test)]
mod tests;
