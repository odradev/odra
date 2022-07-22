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

        impl Into<odra::types::ExecutionError> for #enum_ident {
            fn into(self) -> odra::types::ExecutionError {
                match self {
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

        impl Into<odra::types::OdraError> for #ident {
            fn into(self) -> odra::types::OdraError {
                odra::types::OdraError::ExecutionError(self.into())
            }
        }
    }
}
