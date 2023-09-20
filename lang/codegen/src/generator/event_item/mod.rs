use derive_more::From;
use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;
use quote::quote;

use crate::GenerateCode;

use super::common;

const EVENT_PREFIX: &str = "event_";

#[derive(From)]
pub struct EventItem<'a> {
    event: &'a IrEventItem
}

impl GenerateCode for EventItem<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.event.struct_ident();

        let struct_serialization_code = serialize_struct(self.event);
        let event_def = to_event_def(self.event);

        quote! {
            impl odra::types::event::OdraEvent for #struct_ident {
                fn emit(self) {
                    odra::contract_env::emit_event(self);
                }

                fn name() -> odra::prelude::string::String {
                    odra::prelude::string::String::from(stringify!(#struct_ident))
                }

                #[cfg(not(target_arch = "wasm32"))]
                fn schema() -> odra::types::contract_def::Event {
                    #event_def
                }
            }

            #struct_serialization_code
        }
    }
}

fn to_event_def(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event
        .fields_iter()
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let is_slice = matches!(ty, syn::Type::Slice(syn::TypeSlice { .. }));
            quote! {
                odra::types::contract_def::Argument {
                    ident: odra::prelude::string::String::from(stringify!(#field_ident)),
                    ty: <#ty as odra::types::CLTyped>::cl_type(),
                    is_ref: false,
                    is_slice: #is_slice
                },
            }
        })
        .collect::<TokenStream>();
    quote! {
        odra::types::contract_def::Event {
            ident: odra::prelude::string::String::from(stringify!(#struct_ident)),
            args: odra::prelude::vec![#fields]
        }
    }
}

fn serialize_struct(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event
        .fields_iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    common::serialize_struct(EVENT_PREFIX, struct_ident, &fields)
}
