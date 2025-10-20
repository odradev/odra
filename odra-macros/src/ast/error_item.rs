use crate::ast::fn_utils::SingleArgFnItem;
use crate::ast::utils::{ImplItem, Named};
use crate::ir::{TypeIR, TypeKind};
use crate::utils;
use crate::utils::misc::AsBlock;
use derive_try_from_ref::TryFromRef;
use syn::parse_quote;

use super::schema::SchemaErrorItem;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(TypeIR)]
#[err(syn::Error)]
pub struct OdraErrorAttrItem {
    item: OdraErrorItem,
    schema: SchemaErrorItem
}

#[derive(syn_derive::ToTokens)]
pub struct OdraErrorItem {
    item: syn::Item,
    wasm32_fn: FromErrorItem,
    not_wasm32_fn: FromErrorItem
}

impl TryFrom<&'_ TypeIR> for OdraErrorItem {
    type Error = syn::Error;

    fn try_from(ty: &TypeIR) -> Result<Self, Self::Error> {
        let kind: TypeKind = ty.kind()?;
        let variants = match kind {
            TypeKind::Enum { variants } => variants,
            TypeKind::UnitEnum { variants } => variants,
            _ => {
                return Err(syn::Error::new_spanned(
                    ty.self_code(),
                    "Expected an enum or unit enum for #[odra_error]"
                ))
            }
        }
        .iter()
        .map(|v| v.ident.clone())
        .collect::<Vec<_>>();

        let ident = ty.name()?;
        let ident_error = utils::ident::error();
        let ty_odra_error = utils::ty::odra_error();
        Ok(Self {
            item: ty.self_code().clone(),
            wasm32_fn: FromErrorItem {
                wasm32_attr: utils::attr::wasm32(),
                attr: utils::attr::automatically_derived(),
                impl_item: ImplItem::from(ty, &ty_odra_error)?,
                braces: Default::default(),
                fn_item: SingleArgFnItem::new(
                    &utils::ident::from(),
                    parse_quote!(#ident_error: #ident),
                    utils::misc::ret_ty(&utils::ty::_Self()),
                    utils::expr::user_error(&ident_error).as_block()
                )
            },
            not_wasm32_fn: FromErrorItem {
                wasm32_attr: utils::attr::not_wasm32(),
                attr: utils::attr::automatically_derived(),
                impl_item: ImplItem::from(ty, &ty_odra_error)?,
                braces: Default::default(),
                fn_item: SingleArgFnItem::new(
                    &utils::ident::from(),
                    parse_quote!(#ident_error: #ident),
                    utils::misc::ret_ty(&utils::ty::_Self()),
                    parse_quote!({
                        match error {
                            #(#ident::#variants => OdraError::user(error as u16, stringify!(#variants)),)*
                        }
                    })
                )
            }
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct FromErrorItem {
    wasm32_attr: syn::Attribute,
    attr: syn::Attribute,
    impl_item: ImplItem,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    fn_item: SingleArgFnItem
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_odra_error_item() {
        let ty = test_utils::mock::custom_enum();
        let item = OdraErrorItem::try_from(&ty).unwrap();
        let expected = quote::quote! {
            enum MyType {
                /// Description of A
                A = 10,
                /// Description of B
                B,
            }

            #[cfg(target_arch = "wasm32")]
            #[automatically_derived]
            impl ::core::convert::From<MyType> for OdraError {
                fn from(error: MyType) -> Self {
                    OdraError::user(error as u16)
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            #[automatically_derived]
            impl ::core::convert::From<MyType> for OdraError {
                fn from(error: MyType) -> Self {
                    match error {
                        MyType::A => OdraError::user(error as u16, stringify!(A)),
                        MyType::B => OdraError::user(error as u16, stringify!(B)),
                    }
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
