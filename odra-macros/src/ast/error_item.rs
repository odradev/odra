use crate::ast::fn_utils::SingleArgFnItem;
use crate::ast::utils::{ImplItem, Named};
use crate::ir::TypeIR;
use crate::utils;
use crate::utils::misc::AsBlock;
use syn::parse_quote;

#[derive(syn_derive::ToTokens)]
pub struct OdraErrorItem {
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
        let ty = test_utils::mock_enum();
        let item = OdraErrorItem::try_from(&ty).unwrap();
        let expected = quote::quote! {
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
