use syn::parse_quote;

use crate::ast::fn_utils::FnItem;
use crate::ast::utils::ImplItem;
use crate::ir::TypeIR;
use crate::utils::misc::AsBlock;
use crate::{ir::ModuleStructIR, utils};

#[derive(syn_derive::ToTokens)]
pub struct HasEventsImplItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    events_fn: EventsFnsItem
}

impl TryFrom<&'_ ModuleStructIR> for HasEventsImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_item: ImplItem::has_events(ir)?,
            brace_token: Default::default(),
            events_fn: ir.try_into()?
        })
    }
}

impl TryFrom<&'_ TypeIR> for HasEventsImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_item: ImplItem::has_events(ir)?,
            brace_token: Default::default(),
            events_fn: EventsFnsItem::empty()
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct EventsFnsItem {
    events_fn: FnItem,
    event_schemas_fn: FnItem
}

impl EventsFnsItem {
    pub fn empty() -> Self {
        Self {
            events_fn: FnItem::new(
                &utils::ident::events(),
                vec![],
                Self::events_ret_ty(),
                utils::expr::empty_vec().as_block()
            ),
            event_schemas_fn: FnItem::new(
                &utils::ident::event_schemas(),
                vec![],
                Self::schemas_ret_ty(),
                utils::expr::empty_btree_map().as_block()
            )
        }
    }

    fn events_ret_ty() -> syn::ReturnType {
        let ev_ty = utils::ty::event();
        let vec = utils::ty::vec_of(&ev_ty);
        utils::misc::ret_ty(&vec)
    }

    fn schemas_ret_ty() -> syn::ReturnType {
        let string_ty = utils::ty::string();
        let schema_ty = utils::ty::schema();
        let btree = utils::ty::typed_btree_map(&string_ty, &schema_ty);
        utils::misc::ret_ty(&btree)
    }

    fn events_fn(ir: &ModuleStructIR) -> syn::Result<FnItem> {
        let ident_events = utils::ident::events();
        let struct_events_stmt = struct_events_stmt(ir);
        let chain_events_expr = chain_events_expr(ir)?;
        Ok(FnItem::new(
            &ident_events,
            vec![],
            EventsFnsItem::events_ret_ty(),
            parse_quote!({
                #struct_events_stmt
                #chain_events_expr
            })
        ))
    }

    fn event_schemas_fn(ir: &ModuleStructIR) -> Result<FnItem, syn::Error> {
        let ident_events = utils::ident::event_schemas();
        let struct_events_stmt = struct_event_schemas_stmt(ir);
        let chain_events_expr = chain_event_schemas_expr(ir)?;
        Ok(FnItem::new(
            &ident_events,
            vec![],
            EventsFnsItem::schemas_ret_ty(),
            parse_quote!({
                #struct_events_stmt
                #chain_events_expr
            })
        ))
    }
}

impl TryFrom<&'_ ModuleStructIR> for EventsFnsItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            events_fn: Self::events_fn(ir)?,
            event_schemas_fn: Self::event_schemas_fn(ir)?
        })
    }
}

fn struct_events_stmt(ir: &ModuleStructIR) -> syn::Stmt {
    let events_ident = utils::ident::events();

    let struct_events = ir
        .events()
        .iter()
        .map(utils::expr::into_event)
        .collect::<syn::punctuated::Punctuated<_, syn::token::Comma>>();
    let vec = utils::expr::vec(struct_events);
    parse_quote!(let #events_ident = #vec;)
}

fn chain_events_expr(ir: &ModuleStructIR) -> syn::Result<syn::Expr> {
    let ev_ty = utils::ty::event();
    let events_ident = utils::ident::events();
    let fields_events = ir
        .unique_fields_ty()?
        .iter()
        .map(utils::expr::events)
        .map(|expr| quote::quote!(.chain(#expr)))
        .collect::<Vec<_>>();

    Ok(parse_quote!(
        odra::prelude::BTreeSet::<#ev_ty>::new()
            .into_iter()
            .chain(#events_ident)
            #(#fields_events)*
            .collect()
    ))
}

fn struct_event_schemas_stmt(ir: &ModuleStructIR) -> syn::Stmt {
    let result_ident = utils::ident::result();

    let events = ir
        .events()
        .iter()
        .map(|ty| {
            let name = utils::expr::event_instance_name(ty);
            let schema = utils::expr::event_instance_schema(ty);
            quote::quote!((#name, #schema))
        })
        .collect::<syn::punctuated::Punctuated<_, syn::token::Comma>>();
    let iter = utils::expr::vec(events);
    let new_btree_map = utils::expr::btree_from_iter(&iter);
    parse_quote!(let #result_ident = #new_btree_map;)
}

fn chain_event_schemas_expr(ir: &ModuleStructIR) -> syn::Result<syn::Expr> {
    let result_ident = utils::ident::result();
    let fields_events = ir
        .unique_fields_ty()?
        .iter()
        .map(utils::expr::event_schemas)
        .map(|expr| quote::quote!(.chain(#expr)))
        .collect::<Vec<_>>();

    Ok(parse_quote!(
        #result_ident
            .into_iter()
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
        let module = test_utils::mock::module_definition();
        let expected = quote!(
            impl odra::contract_def::HasEvents for CounterPack {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    let events = odra::prelude::vec![
                        <OnTransfer as odra::contract_def::IntoEvent>::into_event(),
                        <OnApprove as odra::contract_def::IntoEvent>::into_event()
                    ];
                    odra::prelude::BTreeSet::<odra::contract_def::Event>::new()
                        .into_iter()
                        .chain(events)
                        .chain(<Mapping<u8, Counter> as odra::contract_def::HasEvents>::events())
                        .chain(<SubModule<Counter> as odra::contract_def::HasEvents>::events())
                        .chain(<Var<u32> as odra::contract_def::HasEvents>::events())
                        .collect()
                }

                fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                    let result = odra::prelude::BTreeMap::from_iter(
                        odra::prelude::vec![
                            (<OnTransfer as odra::casper_event_standard::EventInstance>::name(), <OnTransfer as odra::casper_event_standard::EventInstance>::schema()),
                            (<OnApprove as odra::casper_event_standard::EventInstance>::name(), <OnApprove as odra::casper_event_standard::EventInstance>::schema())
                        ]
                    );
                    result
                        .into_iter()
                        .chain(<Mapping<u8, Counter> as odra::contract_def::HasEvents>::event_schemas())
                        .chain(<SubModule<Counter> as odra::contract_def::HasEvents>::event_schemas())
                        .chain(<Var<u32> as odra::contract_def::HasEvents>::event_schemas())
                        .collect()
                }
            }
        );
        let actual = HasEventsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
