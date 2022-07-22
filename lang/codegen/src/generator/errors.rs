use convert_case::{Case, Casing};
use derive_more::From;
use odra_ir::ErrorEnumItem as IrErrorEnumItem;
use proc_macro2::TokenStream;

use crate::GenerateCode;

#[derive(From)]
pub struct ErrorEnumItem<'a> {
    item: &'a IrErrorEnumItem,
}

impl GenerateCode for ErrorEnumItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let item_enum = self.item;
        let enum_ident = &self.item.ident;
        let arms = &self
            .item
            .variants
            .iter()
            .flat_map(|v| {
                let ident = &v.ident;
                let msg = &v.ident.to_string().to_case(Case::Title);
                let code = &v.expr;
                quote::quote! {
                    #enum_ident::#ident => odra::types::ExecutionError::new(#code, #msg),
                }
            })
            .collect::<TokenStream>();

        quote::quote! {
            #[odra::odra_error]
            #item_enum

            impl Into<odra::types::ExecutionError> for #enum_ident {
                fn into(self) -> odra::types::ExecutionError {
                    match self {
                        #arms
                    }
                }
            }
        }
    }
}

#[derive(From)]
pub struct OdraErrorItem<'a> {
    item: &'a syn::ItemEnum,
}

impl GenerateCode for OdraErrorItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let item_enum = &self.item;
        let ident = &self.item.ident;

        quote::quote! {
            #item_enum

            impl Into<odra::types::OdraError> for #ident {
                fn into(self) -> odra::types::OdraError {
                    odra::types::OdraError::ExecutionError(self.into())
                }
            }
        }
    }
}
