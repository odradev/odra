use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ir::{ModuleStructIR, TypeIR},
    utils
};

/// Emits `impl odra::contract_def::HasEvents for Xxx`: `events()` and `event_schemas()`, which
/// the host and the schema use to know every event the contract can emit.
pub struct HasEventsImplItem {
    ty_ident: Ident,
    sources: EventSources
}

/// Where a type's events come from.
enum EventSources {
    /// An `#[odra::odra_type]` emits nothing.
    None,
    /// A factory emits its one `XxxContractDeployed` event.
    Only(syn::Type),
    /// A module emits the events of `#[odra::module(events = [..])]` plus those of the types of
    /// its fields, each field type counted once.
    Module {
        declared: Vec<syn::Type>,
        field_types: Vec<syn::Type>
    }
}

impl HasEventsImplItem {
    /// The impl of a type that emits only `event_ty`.
    pub(crate) fn only(ty_ident: Ident, event_ty: syn::Type) -> Self {
        Self {
            ty_ident,
            sources: EventSources::Only(event_ty)
        }
    }
}

impl TryFrom<&ModuleStructIR> for HasEventsImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ty_ident: ir.module_ident(),
            sources: EventSources::Module {
                declared: ir.events(),
                field_types: ir.unique_fields_ty()
            }
        })
    }
}

impl TryFrom<&TypeIR> for HasEventsImplItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ty_ident: ir.ident(),
            sources: EventSources::None
        })
    }
}

impl ToTokens for HasEventsImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty_ident = &self.ty_ident;
        let ty_has_events = utils::ty::has_events();
        let ty_event = utils::ty::event();
        let ty_schema = utils::ty::schema();
        let ty_string = utils::ty::string();
        let ty_vec = utils::ty::vec();
        let ty_btree_map = utils::ty::btree_map();
        let ty_event_instance = utils::ty::event_instance();
        let events = utils::ident::events();
        let event_schemas = utils::ident::event_schemas();
        let result = utils::ident::result();

        let into_event = |ty: &syn::Type| quote!(<#ty as odra::contract_def::IntoEvent>::into_event());
        let name_and_schema = |ty: &syn::Type| {
            quote!((<#ty as #ty_event_instance>::name(), <#ty as #ty_event_instance>::schema()))
        };

        let (events_body, event_schemas_body) = match &self.sources {
            EventSources::None => (quote!(#ty_vec::new()), quote!(#ty_btree_map::new())),
            EventSources::Only(ty) => {
                let into_event = into_event(ty);
                let name_and_schema = name_and_schema(ty);
                (
                    quote!(odra::prelude::vec![#into_event]),
                    quote!(#ty_btree_map::from_iter(odra::prelude::vec![#name_and_schema]))
                )
            }
            EventSources::Module {
                declared,
                field_types
            } => {
                let into_events = declared.iter().map(into_event);
                let names_and_schemas = declared.iter().map(name_and_schema);
                let field_events = field_types
                    .iter()
                    .map(|ty| quote!(.chain(<#ty as #ty_has_events>::events())));
                let field_schemas = field_types
                    .iter()
                    .map(|ty| quote!(.chain(<#ty as #ty_has_events>::event_schemas())));
                (
                    quote! {
                        let #events = odra::prelude::vec![#(#into_events),*];
                        odra::prelude::BTreeSet::<#ty_event>::new()
                            .into_iter()
                            .chain(#events)
                            #(#field_events)*
                            .collect()
                    },
                    quote! {
                        let #result = #ty_btree_map::from_iter(odra::prelude::vec![#(#names_and_schemas),*]);
                        #result
                            .into_iter()
                            #(#field_schemas)*
                            .collect()
                    }
                )
            }
        };

        tokens.extend(quote! {
            impl #ty_has_events for #ty_ident {
                fn #events() -> #ty_vec<#ty_event> {
                    #events_body
                }

                fn #event_schemas() -> #ty_btree_map<#ty_string, #ty_schema> {
                    #event_schemas_body
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
