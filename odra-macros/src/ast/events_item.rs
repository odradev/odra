use syn::parse_quote;

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

#[derive(syn_derive::ToTokens)]
pub struct EventsFnItem {
    sig: syn::Signature,
    block: syn::Block
}

impl TryFrom<&'_ StructIR> for EventsFnItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ StructIR) -> Result<Self, Self::Error> {
        let ev_ty = utils::ty::event();
        let struct_events_stmt = struct_events_stmt(struct_ir);
        let chain_events_expr = chain_events_expr(struct_ir)?;
        
        Ok(Self {
            sig: parse_quote!(fn events() -> Vec<#ev_ty>),
            block: parse_quote!({
                #struct_events_stmt
                #chain_events_expr
            })
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
    parse_quote!(let #events_ident = vec![#struct_events];)
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
                fn events() -> Vec<odra::contract_def::Event> {
                    let events = vec![
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
