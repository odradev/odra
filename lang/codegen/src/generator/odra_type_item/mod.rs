use derive_more::From;
use odra_ir::OdraTypeItem as IrOdraTypeItem;
use proc_macro2::TokenStream;
use quote::quote;

use crate::GenerateCode;

mod clone;
mod ty;

#[derive(From)]
pub struct OdraTypeItem<'a> {
    item: &'a IrOdraTypeItem
}

impl GenerateCode for OdraTypeItem<'_> {
    fn generate_code(&self) -> TokenStream {
        let ident = self.item.ident();

        let ty_code = ty::generate_code(self.item);
        let clone_code = clone::generate_code(self.item);

        quote! {
            #ty_code

            #clone_code

            impl odra::OdraItem for #ident {
                fn is_module() -> bool {
                    false
                }
            }
        }
    }
}
