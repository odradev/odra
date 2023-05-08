use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;

use crate::generator::common::casper;

pub fn generate_code(item: &OdraTypeItem) -> TokenStream {
    let struct_ident = item.struct_ident();
    let fields = item
        .fields_iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    casper::serialize_struct("", struct_ident, &fields)
}
