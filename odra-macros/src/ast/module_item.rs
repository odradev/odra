use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::parts_utils::UseSuperItem,
    ir::{EnumeratedTypedField, ModuleStructIR},
    utils
};

/// Emits `mod __xxx_module { impl Module for Xxx { new, env } }`: `new` instantiates every
/// field as a module component at its index (1-based, in declaration order) and keeps the env.
pub struct ModuleModItem {
    mod_ident: Ident,
    module_ident: Ident,
    fields: Vec<EnumeratedTypedField>
}

impl TryFrom<&ModuleStructIR> for ModuleModItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            mod_ident: ir.module_mod_ident(),
            module_ident: ir.module_ident(),
            fields: ir.typed_fields().to_vec()
        })
    }
}

impl ToTokens for ModuleModItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            mod_ident,
            module_ident,
            fields
        } = self;
        let use_super = UseSuperItem;
        let ty_module = utils::ty::module();
        let ty_module_component = utils::ty::module_component();
        let ty_rc_contract_env = utils::ty::rc_contract_env();
        // The `Module::new` parameter and the hidden field share the name.
        let env = utils::ident::prefixed_env();

        let field_idents = fields.iter().map(|f| &f.ident);
        let instances = fields.iter().map(|EnumeratedTypedField { idx, ident, ty }| {
            quote! {
                let #ident = <#ty as #ty_module_component>::instance(
                    odra::prelude::Rc::clone(&#env),
                    #idx
                );
            }
        });

        tokens.extend(quote! {
            mod #mod_ident {
                #use_super

                impl #ty_module for #module_ident {
                    fn new(#env: #ty_rc_contract_env) -> Self {
                        #(#instances)*
                        Self {
                            #(#field_idents,)*
                            #env
                        }
                    }

                    fn env(&self) -> #ty_rc_contract_env {
                        self.#env.clone()
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
