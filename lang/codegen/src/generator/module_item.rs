use derive_more::From;
use proc_macro2::Ident;
use quote::{format_ident, quote, quote_spanned};

use crate::GenerateCode;

#[derive(From)]
pub struct ModuleStruct<'a> {
    pub contract: &'a odra_ir::module::ModuleStruct
}

impl GenerateCode for ModuleStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let events = &self.contract.events;
        let events = events.events.iter().map(|ev| &ev.name).collect::<Vec<_>>();

        let item_struct = &self.contract.item;
        let submodules = item_struct
            .fields
            .iter()
            .filter(|field| field.ident.is_some())
            .filter_map(|f| match &f.ty {
                syn::Type::Path(path) => {
                    let path = &path.path;
                    if path.segments.len() != 0 {
                        let segment = &path.segments.last().unwrap();
                        if segment.ident != "Variable" && segment.ident != "Mapping" {
                            return Some(segment.ident.clone());
                        }
                    }
                    None
                }
                _ => None
            })
            .collect::<Vec<_>>();
        let mappings: Vec<Ident> = vec![format_ident!("String")];
        let struct_ident = &item_struct.ident;
        let span = item_struct.ident.span();
        let instance = match &self.contract.is_instantiable {
            true => quote_spanned!(span => #[derive(odra::Instance)]),
            false => quote!()
        };
        quote! {
            #instance
            #item_struct

            // #[cfg(feature = "casper")]
            impl odra::types::contract_def::HasEvents for #struct_ident {
                fn events() -> Vec<odra::types::contract_def::Event> {
                    let mut events = vec![];
                    #(
                        events.extend(<#events as odra::types::event::OdraEvent>::schema());
                    )*
                    #(
                        events.extend(<#submodules as odra::types::contract_def::HasEvents>::events());
                    )*

                    #(
                        events.extend(<#mappings as odra::OdraItem>::events());
                    )*
                    events
                }
            }
        }
    }
}
