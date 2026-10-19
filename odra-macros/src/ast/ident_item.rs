use syn::parse_quote;

use crate::ir::ModuleStructIR;
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
    ident_fn: IdentFnItem,
    #[syn(in = brace_token)]
    name_fn: Option<NameFnItem>
}

impl TryFrom<&'_ ModuleStructIR> for HasIdentImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            has_ident_ty: utils::ty::has_ident(),
            for_token: Default::default(),
            module_ident: ir.module_ident(),
            brace_token: Default::default(),
            ident_fn: ir.try_into()?,
            name_fn: NameFnItem::for_module(ir)
        })
    }
}

/// `fn contract_name() -> String`, emitted only when `#[odra::module(name = "..")]` was given; otherwise
/// the trait's default (the ident) applies.
#[derive(syn_derive::ToTokens)]
pub struct NameFnItem {
    sig: syn::Signature,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    expr: syn::Expr
}

impl NameFnItem {
    fn for_module(ir: &ModuleStructIR) -> Option<Self> {
        let name = ir.contract_name();
        if name.is_empty() {
            return None;
        }
        let ret_ty = utils::ty::string();
        Some(Self {
            sig: parse_quote!(fn contract_name() -> #ret_ty),
            brace_token: Default::default(),
            expr: utils::expr::string_from(name)
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

impl TryFrom<&'_ ModuleStructIR> for IdentFnItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
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
        let module = test_utils::mock::module_definition();
        let expected = quote!(
            impl odra::contract_def::HasIdent for CounterPack {
                fn ident() -> odra::prelude::string::String {
                    odra::prelude::string::String::from("CounterPack")
                }
                fn contract_name() -> odra::prelude::string::String {
                    odra::prelude::string::String::from("MyCounterPack")
                }
            }
        );
        let actual = HasIdentImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn no_name_attribute_no_name_fn() {
        let module = test_utils::mock::factory_module_definition();
        let expected = quote!(
            impl odra::contract_def::HasIdent for CounterPack {
                fn ident() -> odra::prelude::string::String {
                    odra::prelude::string::String::from("CounterPack")
                }
            }
        );
        let actual = HasIdentImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
