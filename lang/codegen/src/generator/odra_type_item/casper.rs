use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;

use crate::generator::common::casper;

pub fn generate_code(item: &OdraTypeItem) -> TokenStream {
    let ident = item.ident();

    match item {
        OdraTypeItem::Struct(s) => casper::serialize_struct("", ident, s.fields()),
        OdraTypeItem::Enum(e) => casper::serialize_enum(ident, e.variants())
    }
}
