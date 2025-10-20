use crate::ast::events_item::HasEventsImplItem;
use crate::ast::ident_item::HasIdentImplItem;
use crate::ast::module_def::ModuleDefItem;
use crate::ast::module_item::ModuleModItem;
use crate::ast::schema::{SchemaErrorsItem, SchemaEventsItem, SchemaItem};
use crate::ir::ModuleStructIR;
use derive_try_from_ref::TryFromRef;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleStructIR)]
#[err(syn::Error)]
pub struct FactoryModuleStructItem {
    self_code: ModuleDefItem,
    mod_item: ModuleModItem,
    has_ident_item: HasIdentImplItem,
    has_events_item: HasEventsImplItem, // Should generate factory specific events only
    schema_item: SchemaItem,
    schema_events_item: SchemaEventsItem, // Should generate factory specific events only
    schema_errors: SchemaErrorsItem       // Should generate factory specific errors only
}
