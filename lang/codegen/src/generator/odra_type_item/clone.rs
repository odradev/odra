use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma};

pub fn generate_code(item: &OdraTypeItem) -> TokenStream {
    let ident = item.ident();
    let clone = match item {
        OdraTypeItem::Struct(s) => {
            let code = s
                .fields()
                .iter()
                .map(|field| quote!(#field: ::core::clone::Clone::clone(#field)))
                .collect::<Punctuated<TokenStream, Comma>>();
            quote!(#ident {
                #code
            })
        }
        OdraTypeItem::Enum(e) => {
            let code = e
                .variants()
                .iter()
                .map(|variant| {
                    let variant_ident = &variant.ident;
                    quote!(#ident::#variant_ident => #ident::#variant_ident)
                })
                .collect::<Punctuated<TokenStream, Comma>>();
            quote!(match self {
                #code
            })
        }
    };

    quote! {
        impl ::core::clone::Clone for #ident {
            #[inline]
            fn clone(&self) -> #ident {
                #clone
            }
        }
    }
}
