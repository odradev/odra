use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;

use crate::generator::common::casper;

pub fn generate_code(item: &OdraTypeItem) -> TokenStream {
    let struct_ident = item.struct_ident();

    if let Some(data) = item.data_struct() {
        let fields = data
            .fields
            .iter()
            .map(|f| f.ident.clone().unwrap())
            .collect::<Vec<_>>();

        return casper::serialize_struct("", struct_ident, &fields);
    }

    if let Some(data) = item.data_enum() {
        return casper::serialize_enum(struct_ident, data);
    }

    TokenStream::new()
}
