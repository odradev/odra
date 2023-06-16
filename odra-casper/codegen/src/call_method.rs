use odra_types::contract_def::{Entrypoint, EntrypointType, Event};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma};

use crate::{constructor::WasmConstructor, entrypoints_def::ContractEntrypoints, ty::CasperType};

pub struct CallMethod {
    event_schemas: Vec<Event>,
    entry_points: Vec<Entrypoint>,
    ref_path: syn::Path
}

impl CallMethod {
    pub fn new(
        event_schemas: Vec<Event>,
        entry_points: Vec<Entrypoint>,
        ref_path: syn::Path
    ) -> Self {
        Self {
            event_schemas,
            entry_points,
            ref_path
        }
    }

    fn events(&self) -> TokenStream {
        self.event_schemas
            .iter()
            .map(|ev| {
                let ident = &ev.ident;

                let fields = ev
                    .args
                    .iter()
                    .map(|arg| {
                        let field = &arg.ident;
                        let ty = CasperType(&arg.ty);
                        quote!((#field, #ty))
                    })
                    .collect::<Punctuated<TokenStream, Comma>>();

                quote! {
                    odra::casper::utils::build_event(
                        alloc::string::String::from(#ident),
                        alloc::vec![#fields]
                    )
                }
            })
            .collect::<Punctuated<TokenStream, Comma>>()
            .to_token_stream()
    }

    fn constructor_call(&self) -> Option<TokenStream> {
        let constructors = self
            .entry_points
            .iter()
            .filter(|ep| matches!(ep.ty, EntrypointType::Constructor { .. }))
            .collect::<Vec<_>>();

        if constructors.is_empty() {
            return None;
        };

        Some(WasmConstructor(constructors, &self.ref_path).to_token_stream())
    }
}

impl ToTokens for CallMethod {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let event_schemas = self.events();
        let entry_points = ContractEntrypoints(&self.entry_points);
        let constructor_call = self.constructor_call();

        tokens.extend(quote!{
            #[no_mangle]
            fn call() {
                let schemas = alloc::vec![
                    #event_schemas
                ];

                #entry_points

                #[allow(unused_variables)]
                let contract_package_hash = odra::casper::utils::install_contract(entry_points, schemas);

                #constructor_call
            }
        });
    }
}
