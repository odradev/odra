use crate::ast::fn_utils::{FnItem, SelfFnItem, SingleArgFnItem};
use crate::ast::HasEventsImplItem;
use crate::ir::TypeIR;
use crate::utils;
use crate::utils::misc::AsBlock;
use derive_try_from::TryFromRef;
use syn::parse_quote;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(TypeIR)]
pub struct OdraTypeItem {
    from_bytes_impl: FromBytesItem,
    to_bytes_impl: ToBytesItem,
    cl_type_impl: CLTypedItem,
    has_events_impl: HasEventsImplItem
}

#[derive(syn_derive::ToTokens)]
struct FromBytesItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    fn_item: FromBytesFnItem
}

impl TryFrom<&'_ TypeIR> for FromBytesItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_item: ImplItem::from_bytes(ir),
            brace_token: Default::default(),
            fn_item: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ToBytesItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    fn_item: ToBytesFnItem,
    #[syn(in = brace_token)]
    serialized_length_item: SerializedLengthFnItem
}

impl TryFrom<&'_ TypeIR> for ToBytesItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_item: ImplItem::to_bytes(ir),
            brace_token: Default::default(),
            fn_item: ir.try_into()?,
            serialized_length_item: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct CLTypedItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    fn_item: FnItem
}

impl TryFrom<&'_ TypeIR> for CLTypedItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let ty_cl_type_any = utils::ty::cl_type_any();
        let ty_cl_type = utils::ty::cl_type();
        Ok(Self {
            impl_item: ImplItem::cl_typed(ir),
            brace_token: Default::default(),
            fn_item: FnItem::new(
                &utils::ident::cl_type(),
                vec![],
                utils::misc::ret_ty(&ty_cl_type),
                ty_cl_type_any.as_block()
            )
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct FromBytesFnItem {
    fn_item: SingleArgFnItem
}

impl TryFrom<&'_ TypeIR> for FromBytesFnItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let ty_bytes_slice = utils::ty::bytes_slice();
        let ty_self = utils::ty::_Self();
        let ty_ok = parse_quote!((#ty_self, #ty_bytes_slice));
        let ty_ret = utils::ty::bytes_result(&ty_ok);

        let ident_bytes = utils::ident::bytes();
        let ident_from_bytes = utils::ident::from_bytes();

        let from_bytes_expr = utils::expr::failable_from_bytes(&ident_bytes);
        let fields = ir
            .fields()?
            .into_iter()
            .collect::<syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>>();
        let deser = ir.map_fields(|i| quote::quote!(let (#i, #ident_bytes) = #from_bytes_expr;))?;
        let arg = parse_quote!(#ident_bytes: #ty_bytes_slice);
        let ret_ty = utils::misc::ret_ty(&ty_ret);
        let block = parse_quote!({
            #(#deser)*
            Ok((Self { #fields }, #ident_bytes))
        });

        Ok(Self {
            fn_item: SingleArgFnItem::new(&ident_from_bytes, arg, ret_ty, block)
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ToBytesFnItem {
    fn_item: SelfFnItem
}

impl TryFrom<&'_ TypeIR> for ToBytesFnItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let ty_bytes_vec = utils::ty::bytes_vec();
        let ty_ret = utils::ty::bytes_result(&ty_bytes_vec);
        let ty_self = utils::ty::_self();

        let ident_result = utils::ident::result();
        let serialized_length_expr = utils::expr::serialized_length(&ty_self);

        let init_vec_stmt =
            utils::stmt::new_mut_vec_with_capacity(&ident_result, &serialized_length_expr);

        let serialize = ir.map_fields(|i| {
            let member = utils::member::_self(i);
            let expr_to_bytes = utils::expr::failable_to_bytes(&member);
            quote::quote!(#ident_result.extend(#expr_to_bytes);)
        })?;

        let name = utils::ident::to_bytes();
        let ret_ty = utils::misc::ret_ty(&ty_ret);
        let block = parse_quote!({
            #init_vec_stmt
            #(#serialize)*
            Ok(#ident_result)
        });
        Ok(Self {
            fn_item: SelfFnItem::new(&name, ret_ty, block)
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct SerializedLengthFnItem {
    fn_item: SelfFnItem
}

impl TryFrom<&'_ TypeIR> for SerializedLengthFnItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let ty_usize = utils::ty::usize();
        let ident_result = utils::ident::result();

        let stmts = ir.map_fields(|i| {
            let member = utils::member::_self(i);
            let expr = utils::expr::serialized_length(&member);
            let stmt: syn::Stmt = parse_quote!(#ident_result += #expr;);
            stmt
        })?;

        let name = utils::ident::serialized_length();
        let ret_ty = utils::misc::ret_ty(&ty_usize);
        let block = parse_quote!({
            let mut #ident_result = 0;
            #(#stmts)*
            #ident_result
        });
        Ok(Self {
            fn_item: SelfFnItem::new(&name, ret_ty, block)
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ImplItem {
    impl_token: syn::Token![impl],
    ty: syn::Type,
    for_token: syn::Token![for],
    ident: syn::Ident
}

impl ImplItem {
    fn new(ir: &TypeIR, ty: syn::Type) -> Self {
        Self {
            impl_token: Default::default(),
            ty,
            for_token: Default::default(),
            ident: ir.name()
        }
    }

    fn from_bytes(ir: &TypeIR) -> Self {
        Self::new(ir, utils::ty::from_bytes())
    }

    fn to_bytes(ir: &TypeIR) -> Self {
        Self::new(ir, utils::ty::to_bytes())
    }

    fn cl_typed(ir: &TypeIR) -> Self {
        Self::new(ir, utils::ty::cl_typed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn test_odra_type() {
        let ir = test_utils::mock_struct();
        let item = OdraTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            impl odra::FromBytes for MyType {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::BytesReprError> {
                    let (a, bytes) = odra::FromBytes::from_bytes(bytes)?;
                    let (b, bytes) = odra::FromBytes::from_bytes(bytes)?;

                    Ok((Self {
                        a,
                        b
                    }, bytes))
                }
            }

            impl odra::ToBytes for MyType {
                fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::BytesReprError> {
                    let mut result = odra::prelude::vec::Vec::with_capacity(self.serialized_length());
                    result.extend(odra::ToBytes::to_bytes(&self.a)?);
                    result.extend(odra::ToBytes::to_bytes(&self.b)?);
                    Ok(result)
                }

                fn serialized_length(&self) -> usize {
                    let mut result = 0;
                    result += self.a.serialized_length();
                    result += self.b.serialized_length();
                    result
                }
            }

            impl odra::casper_types::CLTyped for MyType {
                fn cl_type() -> odra::casper_types::CLType {
                    odra::casper_types::CLType::Any
                }
            }

            impl odra::contract_def::HasEvents for MyType {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    odra::prelude::vec::Vec::new()
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }
}
