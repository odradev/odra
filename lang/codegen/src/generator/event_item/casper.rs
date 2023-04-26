use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;

use crate::generator::common::casper;

const EVENT_PREFIX: &str = "event_";

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event
        .fields_iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    casper::serialize_struct(EVENT_PREFIX, struct_ident, &fields)
}
