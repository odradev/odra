use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{ast::parts_utils::UseSuperItem, ir::ModuleImplIR, utils};

/// Emits `mod __xxx_schema { fn module_schema() -> ContractBlueprint }`, compiled only when the
/// crate is built for this module: the symbol `cargo odra` calls to read the contract's
/// blueprint out of the host build.
pub struct BlueprintItem {
    module_name: String,
    mod_ident: Ident,
    module_ident: Ident
}

impl TryFrom<&ModuleImplIR> for BlueprintItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_name: module.module_str(),
            mod_ident: module.schema_mod_ident(),
            module_ident: module.module_ident()
        })
    }
}

impl ToTokens for BlueprintItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            module_name,
            mod_ident,
            module_ident
        } = self;
        let odra_module_attr = utils::attr::odra_module(module_name);
        let use_super = UseSuperItem;

        tokens.extend(quote! {
            #odra_module_attr
            mod #mod_ident {
                #use_super

                #[no_mangle]
                #[cfg(not(target_arch = "wasm32"))]
                fn module_schema() -> odra::contract_def::ContractBlueprint {
                    odra::contract_def::ContractBlueprint::new::<#module_ident>()
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
