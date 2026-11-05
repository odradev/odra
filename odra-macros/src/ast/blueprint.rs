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
        let not_wasm32_attr = utils::attr::not_wasm32();
        let no_mangle_attr = utils::attr::no_mangle();
        let use_super = UseSuperItem;
        let module_schema = utils::ident::module_schema();
        let ty_blueprint = utils::ty::contract_blueprint();

        tokens.extend(quote! {
            #odra_module_attr
            mod #mod_ident {
                #use_super

                #no_mangle_attr
                #not_wasm32_attr
                fn #module_schema() -> #ty_blueprint {
                    #ty_blueprint::new::<#module_ident>()
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
