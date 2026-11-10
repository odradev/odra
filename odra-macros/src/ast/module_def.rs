use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;

use crate::ir::ModuleStructIR;
use crate::utils;

/// Emits the user's module struct with the hidden `__odra_env: Rc<ContractEnv>` field appended,
/// which `Module::new` fills and `env()` reads.
pub struct ModuleDefItem {
    item_struct: syn::ItemStruct
}

impl ToTokens for ModuleDefItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item_struct.to_tokens(tokens);
    }
}

impl TryFrom<&ModuleStructIR> for ModuleDefItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        let mut item_struct = ir.self_code().clone();
        let env = utils::ident::prefixed_env();
        let env_field = syn::Field::parse_named
            .parse2(quote::quote!(#env: odra::prelude::Rc<odra::ContractEnv>))?;

        let fields = item_struct
            .fields
            .into_iter()
            .chain(vec![env_field])
            .collect::<syn::punctuated::Punctuated<_, _>>();

        item_struct.fields = syn::Fields::Named(syn::FieldsNamed {
            brace_token: Default::default(),
            named: fields
        });

        Ok(Self { item_struct })
    }
}

#[cfg(test)]
mod tests;
