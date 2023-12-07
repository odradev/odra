use syn::parse_quote;

use crate::ir::StructIR;
use crate::utils;

#[derive(syn_derive::ToTokens)]
pub struct HasIdentImplItem {
    impl_token: syn::token::Impl,
    has_ident_ty: syn::Type,
    for_token: syn::token::For,
    module_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    ident_fn: IdentFnItem
}

impl TryFrom<&'_ StructIR> for HasIdentImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ StructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            has_ident_ty: utils::ty::has_ident(),
            for_token: Default::default(),
            module_ident: ir.module_ident(),
            brace_token: Default::default(),
            ident_fn: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct IdentFnItem {
    sig: syn::Signature,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    expr: syn::Expr
}

impl TryFrom<&'_ StructIR> for IdentFnItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ StructIR) -> Result<Self, Self::Error> {
        let ident = utils::ident::ident();
        let ret_ty = utils::ty::string();

        Ok(Self {
            sig: parse_quote!(fn #ident() -> #ret_ty),
            brace_token: Default::default(),
            expr: utils::expr::string_from(ir.module_str())
        })
    }
}

#[cfg(test)]
mod test {
    use crate::ast::ident_item::HasIdentImplItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn test_entrypoints() {
        let module = test_utils::mock_module_definition();
        let expected = quote!(
            impl odra::contract_def::HasIdent for CounterPack {
                fn ident() -> String {
                    String::from("CounterPack")
                }
            }
        );
        let actual = HasIdentImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
