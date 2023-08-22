use derive_more::From;
use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;
use quote::quote;

use crate::GenerateCode;

mod casper;
mod mock_vm;

#[derive(From)]
pub struct EventItem<'a> {
    event: &'a IrEventItem
}

impl GenerateCode for EventItem<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.event.struct_ident();

        let casper_code = casper::generate_code(self.event);
        let mock_vm_code = mock_vm::generate_code(self.event);
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

            #casper_code

            #mock_vm_code
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
            quote! {
                odra::types::contract_def::Argument {
                    ident: odra::prelude::string::String::from(stringify!(#field_ident)),
                    ty: <#ty as odra::types::Typed>::ty(),
                    is_ref: false,
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
