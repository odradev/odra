use convert_case::{Case, Casing};
use odra_ir::ErrorEnumItem;
use proc_macro2::TokenStream;

pub fn generate_error_enum(item_enum: ErrorEnumItem) -> TokenStream {
    let enum_ident = &item_enum.ident;
    let arms = &item_enum
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

        impl From<#enum_ident> for odra::types::ExecutionError {
            fn from(value: #enum_ident) -> Self {
                match value {
                    #arms
                }
            }
        }
    }
}

pub fn generate_into_odra_error(item_enum: syn::ItemEnum) -> TokenStream {
    let ident = &item_enum.ident;

    quote::quote! {
        #item_enum

        impl From<#ident> for odra::types::OdraError {
            fn from(value: #ident) -> Self {
                odra::types::OdraError::ExecutionError(value.into())
            }
        }
    }
}
