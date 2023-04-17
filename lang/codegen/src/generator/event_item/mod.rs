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

        quote! {
            impl odra::types::event::OdraEvent for #struct_ident {
                fn emit(self) {
                    odra::contract_env::emit_event(self);
                }

                fn name() -> String {
                    String::from(stringify!(#struct_ident))
                }

                fn schema() -> Vec<odra::types::contract_def::Event> {
                    vec![]
                }
            }

            #casper_code

            #mock_vm_code
        }
    }
}
