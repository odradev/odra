use derive_more::From;
use odra_ir::OdraTypeItem as IrOdraTypeItem;
use proc_macro2::TokenStream;
use quote::quote;

use crate::GenerateCode;

mod casper;
mod mock_vm;

#[derive(From)]
pub struct OdraTypeItem<'a> {
    item: &'a IrOdraTypeItem
}

impl GenerateCode for OdraTypeItem<'_> {
    fn generate_code(&self) -> TokenStream {
        let casper_code = casper::generate_code(self.item);
        let mock_vm_code = mock_vm::generate_code(self.item);

        quote! {
            #casper_code

            #mock_vm_code
        }
    }
}
