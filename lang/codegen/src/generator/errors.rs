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
                    #enum_ident::#ident => odra_types::ExecutionError::new(#code, #msg),
                }
            })
            .collect::<TokenStream>();

        quote::quote! {
            #[odra_proc_macros::odra_error]
            #item_enum

            impl From<#enum_ident> for odra_types::ExecutionError {
                fn from(value: #enum_ident) -> Self {
                    match value {
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

            impl From<#ident> for odra_types::OdraError {
                fn from(value: #ident) -> Self {
                    odra_types::OdraError::ExecutionError(value.into())
                }
            }
        }
    }
}
