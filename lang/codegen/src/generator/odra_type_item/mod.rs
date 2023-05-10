use derive_more::From;
use odra_ir::OdraTypeItem as IrOdraTypeItem;
use proc_macro2::TokenStream;
use quote::quote;

use crate::GenerateCode;

mod casper;
mod clone;
mod mock_vm;

#[derive(From)]
pub struct OdraTypeItem<'a> {
    item: &'a IrOdraTypeItem
}

impl GenerateCode for OdraTypeItem<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.item.struct_ident();

        let casper_code = casper::generate_code(self.item);
        let mock_vm_code = mock_vm::generate_code(self.item);
        let clone_code = clone::generate_code(self.item, &struct_ident);

        quote! {
            #casper_code

            #mock_vm_code

            #clone_code

            impl odra::OdraItem for #struct_ident {
                fn is_module() -> bool {
                    false
                }
            }
        }
    }
}
