use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;

use crate::generator::common;

pub fn generate_code(item: &OdraTypeItem) -> TokenStream {
    let struct_ident = item.struct_ident();

    if let Some(data) = item.data_struct() {
        let fields = data
            .fields
            .iter()
            .map(|f| f.ident.clone().unwrap())
            .collect::<Vec<_>>();

        return common::mock_vm::serialize_struct(struct_ident, &fields);
    }

    if let Some(data) = item.data_enum() {
        return common::mock_vm::serialize_enum(struct_ident, data);
    }

    TokenStream::new()
}
