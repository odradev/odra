use crate::ast::factory::parts::{FactoryEventItem, FactoryHasEventsImplItem};
use crate::ast::ident_item::HasIdentImplItem;
use crate::ast::module_def::ModuleDefItem;
use crate::ast::module_item::ModuleModItem;
use crate::ast::schema::{FactorySchemaErrorsItem, FactorySchemaEventsItem, SchemaItem};
use crate::ir::ModuleStructIR;
use derive_try_from_ref::TryFromRef;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleStructIR)]
#[err(syn::Error)]
pub struct FactoryModuleStructItem {
    self_code: ModuleDefItem,
    mod_item: ModuleModItem,
    has_ident_item: HasIdentImplItem,
    has_events_item: FactoryEventItem,
    factory_has_events_impl_item: FactoryHasEventsImplItem,
    schema_item: SchemaItem,
    schema_events_item: FactorySchemaEventsItem,
    schema_errors: FactorySchemaErrorsItem       // Should generate factory specific errors only
}
