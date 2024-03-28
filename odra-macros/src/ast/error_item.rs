use crate::ast::fn_utils::SingleArgFnItem;
use crate::ast::utils::{ImplItem, Named};
use crate::ir::TypeIR;
use crate::utils;
use crate::utils::misc::AsBlock;
use syn::parse_quote;
use derive_try_from_ref::TryFromRef;

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
    attr: syn::Attribute,
    impl_item: ImplItem,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    fn_item: SingleArgFnItem
}

impl TryFrom<&'_ TypeIR> for OdraErrorItem {
    type Error = syn::Error;

    fn try_from(ty: &TypeIR) -> Result<Self, Self::Error> {
        let ident = ty.name()?;
        let ident_error = utils::ident::error();
        let ty_odra_error = utils::ty::odra_error();
        Ok(Self {
            item: ty.self_code().clone(),
            attr: utils::attr::automatically_derived(),
            impl_item: ImplItem::from(ty, &ty_odra_error)?,
            braces: Default::default(),
            fn_item: SingleArgFnItem::new(
                &utils::ident::from(),
                parse_quote!(#ident_error: #ident),
                utils::misc::ret_ty(&utils::ty::_Self()),
                utils::expr::user_error(&ident_error).as_block()
            )
        })
    }
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
            
            #[automatically_derived]
            impl ::core::convert::From<MyType> for odra::OdraError {
                fn from(error: MyType) -> Self {
                    odra::OdraError::user(error as u16)
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
