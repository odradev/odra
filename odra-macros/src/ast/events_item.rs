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
        let result = utils::ident::result();

        let into_event = |ty: &syn::Type| quote!(<#ty as odra::contract_def::IntoEvent>::into_event());
        // `event_schemas()` is one map filled with `insert` for the type's own events and `extend`
        // for the fields' maps. Building it with `BTreeMap::from_iter` over a chain of the fields'
        // iterators made rustc instantiate a sort for every module (about 7 KB of wasm each);
        // `insert` and `extend` reuse the code the schema migration needs anyway.
        let insert_schema = |ty: &syn::Type| {
            quote!(#result.insert(<#ty as odra::casper_event_standard::EventInstance>::name(), <#ty as odra::casper_event_standard::EventInstance>::schema());)
        };

        let (events_body, event_schemas_body) = match &self.sources {
            EventSources::None => (quote!(odra::prelude::vec::Vec::new()), quote!(odra::prelude::BTreeMap::new())),
            EventSources::Only(ty) => {
                let into_event = into_event(ty);
                let insert = insert_schema(ty);
                (
                    quote!(odra::prelude::vec![#into_event]),
                    quote! {
                        let mut #result = odra::prelude::BTreeMap::new();
                        #insert
                        #result
                    }
                )
            }
            EventSources::Module {
                declared,
                field_types
            } => {
                let into_events = declared.iter().map(into_event);
                let field_events = field_types
                    .iter()
                    .map(|ty| quote!(.chain(<#ty as odra::contract_def::HasEvents>::events())));
                let inserts = declared.iter().map(insert_schema);
                let extends = field_types
                    .iter()
                    .map(|ty| quote!(#result.extend(<#ty as odra::contract_def::HasEvents>::event_schemas());));
                let schema_stmts = inserts.chain(extends).collect::<Vec<_>>();
                let event_schemas_body = if schema_stmts.is_empty() {
                    quote!(odra::prelude::BTreeMap::new())
                } else {
                    quote! {
                        let mut #result = odra::prelude::BTreeMap::new();
                        #(#schema_stmts)*
                        #result
                    }
                };
                (
                    quote! {
                        let events = odra::prelude::vec![#(#into_events),*];
                        odra::prelude::BTreeSet::<odra::contract_def::Event>::new()
                            .into_iter()
                            .chain(events)
                            #(#field_events)*
                            .collect()
                    },
                    event_schemas_body
                )
            }
        };

        tokens.extend(quote! {
            impl odra::contract_def::HasEvents for #ty_ident {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    #events_body
                }

                fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                    #event_schemas_body
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
