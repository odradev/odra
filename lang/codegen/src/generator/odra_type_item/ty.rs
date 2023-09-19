use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;

use crate::generator::common;

pub fn generate_code(item: &OdraTypeItem) -> TokenStream {
    let ident = item.ident();

    match item {
        OdraTypeItem::Struct(s) => common::serialize_struct("", ident, s.fields()),
        OdraTypeItem::Enum(e) => common::serialize_enum(ident, e.variants())
    }
}
