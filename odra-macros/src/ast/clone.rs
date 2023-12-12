use crate::ast::fn_utils::SelfFnItem;
use crate::ast::utils::{ImplItem, Named};
use crate::ir::TypeIR;
use crate::utils;
use crate::utils::misc::AsBlock;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma};

#[derive(syn_derive::ToTokens)]
pub struct CloneItem {
    attr: syn::Attribute,
    impl_item: ImplItem,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    inline_attr: syn::Attribute,
    #[syn(in = braces)]
    fn_item: SelfFnItem
}

impl TryFrom<&'_ TypeIR> for CloneItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let ident_clone = utils::ident::clone();
        let ident = ir.name()?;
        let fields = ir.fields()?;
        let block = match ir.is_enum() {
            true => {
                let variants = fields
                    .iter()
                    .map(|field| map_enum_ident(&ident, field))
                    .collect::<Punctuated<TokenStream, Comma>>();
                let ty_self = utils::ty::_self();
                quote!(match #ty_self { #variants }).as_block()
            }
            false => {
                let variants = fields
                    .iter()
                    .map(|field| map_struct_ident(&ident, field))
                    .collect::<Punctuated<TokenStream, Comma>>();
                let ty_self = utils::ty::_Self();
                quote!(#ty_self { #variants }).as_block()
            }
        };

        Ok(Self {
            attr: utils::attr::automatically_derived(),
            impl_item: ImplItem::clone(ir)?,
            braces: Default::default(),
            inline_attr: utils::attr::inline(),
            fn_item: SelfFnItem::new(&ident_clone, ret_ty(), block)
        })
    }
}

fn map_struct_ident(_ident: &syn::Ident, field: &syn::Ident) -> TokenStream {
    let ty_self = utils::ty::self_ref();
    quote!(#field: ::core::clone::Clone::clone(#ty_self.#field))
}

fn map_enum_ident(ident: &syn::Ident, variant: &syn::Ident) -> TokenStream {
    quote!(#ident::#variant => #ident::#variant)
}

fn ret_ty() -> syn::ReturnType {
    utils::misc::ret_ty(&utils::ty::_Self())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_struct() {
        let ir = test_utils::mock_struct();
        let item = CloneItem::try_from(&ir).unwrap();
        let expected = quote! {
            #[automatically_derived]
            impl ::core::clone::Clone for MyType {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        a: ::core::clone::Clone::clone(&self.a),
                        b: ::core::clone::Clone::clone(&self.b),
                    }
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_enum() {
        let ir = test_utils::mock_enum();
        let item = CloneItem::try_from(&ir).unwrap();
        let expected = quote! {
            #[automatically_derived]
            impl ::core::clone::Clone for MyType {
                #[inline]
                fn clone(&self) -> Self {
                    match self {
                        MyType::A => MyType::A,
                        MyType::B => MyType::B,
                    }
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
