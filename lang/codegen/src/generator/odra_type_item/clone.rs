use odra_ir::OdraTypeItem;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma};

pub fn generate_code(item: &OdraTypeItem, struct_ident: &Ident) -> TokenStream {
    let clone_struct = item.data_struct().map(|data| {
        data.fields
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                quote!(#field_ident: ::core::clone::Clone::clone(&self.#field_ident))
            })
            .collect::<Punctuated<TokenStream, Comma>>()
    });

    let clone_enum = item
        .data_enum()
        .map(|data|
            data.variants
            .iter()
            .map(|variant| {
                let ident = &variant.ident;
                let cloned_fields = variant.fields.iter().enumerate().map(|(idx, f)| {
                    let field_ident = match &f.ident {
                        Some(ident) => ident.clone(),
                        None => format_ident!("f{}", idx),
                    };
                    quote!(::core::clone::Clone::clone(#field_ident))
                }).collect::<Vec<_>>();

                let fields = variant.fields.iter().enumerate().map(|(idx, f)| {
                    let field_ident = match &f.ident {
                        Some(ident) => ident.clone(),
                        None => format_ident!("f{}", idx),
                    };
                    quote!(#field_ident)
                }).collect::<Vec<_>>();
                match &variant.fields {
                    syn::Fields::Named(_) => quote! {
                        #struct_ident::#ident { #(#fields),* } => #struct_ident::#ident {
                            #(#fields: #cloned_fields),*
                        }
                    },
                    syn::Fields::Unnamed(_) => quote! {
                        #struct_ident::#ident(#(#fields),*) => #struct_ident::#ident( #(#cloned_fields),*)
                    },
                    syn::Fields::Unit => quote!(#struct_ident::#ident => #struct_ident::#ident)
                }
            })
            .collect::<Punctuated<TokenStream, Comma>>());

    let mut clone = TokenStream::new();
    if let Some(code) = clone_enum {
        clone = quote!(match self {
            #code
        });
    }

    if let Some(code) = clone_struct {
        clone = quote!(#struct_ident {
            #code
        });
    }

    quote! {
        impl ::core::clone::Clone for #struct_ident {
            #[inline]
            fn clone(&self) -> #struct_ident {
                #clone
            }
        }
    }
}
