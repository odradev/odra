use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;

use crate::generator::common::casper;

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    casper::serialize_struct(struct_ident, fields)
}
