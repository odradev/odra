use derive_try_from_ref::TryFromRef;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::ast::factory::parts::{FactoryEventItem, FactoryHasEventsImplItem};
use crate::ast::ident_item::HasIdentImplItem;
use crate::ast::module_def::ModuleDefItem;
use crate::ast::module_item::ModuleModItem;
use crate::ast::parts_utils::OdraModuleCfgItem;
use crate::ast::schema::{
    FactorySchemaErrorsItem, FactorySchemaEventsItem, SchemaItem, SchemaStorageLayoutItem
};
use crate::ir::ModuleStructIR;

/// Emits everything `#[odra::factory]` adds to a factory struct, in this order.
#[derive(TryFromRef)]
#[source(ModuleStructIR)]
#[err(syn::Error)]
pub struct FactoryModuleStructItem {
    self_code: ModuleDefItem,
    mod_item: ModuleModItem,
    has_ident_item: HasIdentImplItem,
    has_events_item: FactoryEventItem,
    factory_has_events_impl_item: FactoryHasEventsImplItem,
    schema_item: OdraModuleCfgItem<SchemaItem>,
    schema_events_item: FactorySchemaEventsItem,
    schema_errors: FactorySchemaErrorsItem,
    schema_storage_layout: SchemaStorageLayoutItem
}

impl ToTokens for FactoryModuleStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            self_code,
            mod_item,
            has_ident_item,
            has_events_item,
            factory_has_events_impl_item,
            schema_item,
            schema_events_item,
            schema_errors,
            schema_storage_layout
        } = self;
        tokens.extend(quote! {
            #self_code
            #mod_item
            #has_ident_item
            #has_events_item
            #factory_has_events_impl_item
            #schema_item
            #schema_events_item
            #schema_errors
            #schema_storage_layout
        });
    }
}
