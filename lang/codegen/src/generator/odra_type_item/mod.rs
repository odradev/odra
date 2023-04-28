use derive_more::From;
use odra_ir::OdraTypeItem as IrOdraTypeItem;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma};

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

        let struct_ident = self.item.struct_ident();

        let clone_fields = self.item.fields_iter().map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            quote! {
                #field_ident: ::core::clone::Clone::clone(&self.#field_ident)
            }
        }).collect::<Punctuated<TokenStream, Comma>>();

        quote! {
            #casper_code

            #mock_vm_code

            impl ::core::clone::Clone for #struct_ident {
                #[inline]
                fn clone(&self) -> #struct_ident {
                    #struct_ident {
                        #clone_fields
                    }
                }
            }

            impl odra::OdraItem for #struct_ident {
                fn is_module() -> bool {
                    false
                }
            }
        }
    }
}
