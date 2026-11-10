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

impl TryFrom<&ModuleStructIR> for FactoryModuleStructItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            self_code: ModuleDefItem::try_from(ir)?,
            mod_item: ModuleModItem::try_from(ir)?,
            has_ident_item: HasIdentImplItem::try_from(ir)?,
            has_events_item: FactoryEventItem::try_from(ir)?,
            factory_has_events_impl_item: FactoryHasEventsImplItem::try_from(ir)?,
            schema_item: OdraModuleCfgItem::try_from(ir)?,
            schema_events_item: FactorySchemaEventsItem::try_from(ir)?,
            schema_errors: FactorySchemaErrorsItem::try_from(ir)?,
            schema_storage_layout: SchemaStorageLayoutItem::try_from(ir)?
        })
    }
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
