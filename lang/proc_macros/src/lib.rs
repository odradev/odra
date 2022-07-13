extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod event;
mod external_contract;
mod instance;
mod module;

#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::generate_code(attr, item).into()
}

#[proc_macro_attribute]
pub fn instance(attr: TokenStream, item: TokenStream) -> TokenStream {
    instance::generate_code(attr, item).into()
}

#[proc_macro_attribute]
pub fn external_contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    external_contract::generate_code(attr, item).into()
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    event::generate_code(parse_macro_input!(input as DeriveInput)).into()
}
