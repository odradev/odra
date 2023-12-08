use syn::parse_quote;

use crate::ast::fn_utils::FnItem;
use crate::ast::utils::Named;
use crate::ir::TypeIR;
use crate::utils::misc::AsBlock;
use crate::{ir::StructIR, utils};

#[derive(syn_derive::ToTokens)]
pub struct HasEventsImplItem {
    impl_token: syn::token::Impl,
    has_ident_ty: syn::Type,
    for_token: syn::token::For,
    module_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    events_fn: EventsFnItem
}

impl TryFrom<&'_ StructIR> for HasEventsImplItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ StructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            has_ident_ty: utils::ty::has_events(),
            for_token: Default::default(),
            module_ident: struct_ir.module_ident(),
            brace_token: Default::default(),
            events_fn: struct_ir.try_into()?
        })
    }
}

impl TryFrom<&'_ TypeIR> for HasEventsImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            has_ident_ty: utils::ty::has_events(),
            for_token: Default::default(),
            module_ident: ir.name()?,
            brace_token: Default::default(),
            events_fn: EventsFnItem::empty()
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct EventsFnItem {
    fn_item: FnItem
}

impl EventsFnItem {
    pub fn empty() -> Self {
        let ident_events = utils::ident::events();
        let empty_vec = utils::expr::empty_vec();
        Self {
            fn_item: FnItem::new(&ident_events, vec![], Self::ret_ty(), empty_vec.as_block())
        }
    }

    fn ret_ty() -> syn::ReturnType {
        let ev_ty = utils::ty::event();
        let vec = utils::ty::vec_of(&ev_ty);
        utils::misc::ret_ty(&vec)
    }
}

impl TryFrom<&'_ StructIR> for EventsFnItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ StructIR) -> Result<Self, Self::Error> {
        let ident_events = utils::ident::events();
        let struct_events_stmt = struct_events_stmt(struct_ir);
        let chain_events_expr = chain_events_expr(struct_ir)?;
        Ok(Self {
            fn_item: FnItem::new(
                &ident_events,
                vec![],
                Self::ret_ty(),
                parse_quote!({
                    #struct_events_stmt
                    #chain_events_expr
                })
            )
        })
    }
}

fn struct_events_stmt(ir: &StructIR) -> syn::Stmt {
    let events_ident = utils::ident::events();

    let struct_events = ir
        .events()
        .iter()
        .map(utils::expr::into_event)
        .collect::<syn::punctuated::Punctuated<_, syn::token::Comma>>();
    let vec = utils::expr::vec(struct_events);
    parse_quote!(let #events_ident = #vec;)
}

fn chain_events_expr(ir: &StructIR) -> Result<syn::Expr, syn::Error> {
    let ev_ty = utils::ty::event();
    let events_ident = utils::ident::events();
    let fields_events = ir
        .unique_fields_ty()?
        .iter()
        .map(utils::expr::events)
        .map(|expr| quote::quote!(.chain(#expr)))
        .collect::<Vec<_>>();

    Ok(parse_quote!(
        BTreeSet::<#ev_ty>::new()
            .into_iter()
            .chain(#events_ident)
            #(#fields_events)*
            .collect()
    ))
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::HasEventsImplItem;

    #[test]
    fn counter_pack() {
        let module = test_utils::mock_module_definition();
        let expected = quote!(
            impl odra::contract_def::HasEvents for CounterPack {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    let events = odra::prelude::vec![
                        <OnTransfer as odra::contract_def::IntoEvent>::into_event(),
                        <OnApprove as odra::contract_def::IntoEvent>::into_event()
                    ];
                    BTreeSet::<odra::contract_def::Event>::new()
                        .into_iter()
                        .chain(events)
                        .chain(<Mapping<u8, Counter> as odra::contract_def::HasEvents>::events())
                        .chain(<ModuleWrapper<Counter> as odra::contract_def::HasEvents>::events())
                        .chain(<Variable<u32> as odra::contract_def::HasEvents>::events())
                        .collect()
                }
            }
        );
        let actual = HasEventsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
