use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::ir::{errors, ModuleStructIR};
use crate::utils;

const MAX_FIELDS: usize = 255;

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
        let env_field: syn::Field = utils::misc::field(
            &utils::ident::prefixed_env(),
            &utils::ty::rc_contract_env()
        );

        if item_struct.fields.len() > MAX_FIELDS {
            return Err(errors::module_too_many_fields(&item_struct.fields, MAX_FIELDS));
        }

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
