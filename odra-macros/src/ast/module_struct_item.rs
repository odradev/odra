use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::ast::events_item::HasEventsImplItem;
use crate::ast::factory::FactoryModuleItem;
use crate::ast::ident_item::HasIdentImplItem;
use crate::ast::module_def::ModuleDefItem;
use crate::ast::module_item::ModuleModItem;
use crate::ast::parts_utils::OdraModuleCfgItem;
use crate::ast::schema::{SchemaErrorsItem, SchemaEventsItem, SchemaItem, SchemaStorageLayoutItem};
use crate::ir::ModuleStructIR;

/// Emits everything `#[odra::module]` adds to a module struct, in this order.
pub struct ModuleStructItem {
    self_code: ModuleDefItem,
    mod_item: ModuleModItem,
    has_ident_item: HasIdentImplItem,
    has_events_item: HasEventsImplItem,
    schema_item: OdraModuleCfgItem<SchemaItem>,
    schema_events_item: SchemaEventsItem,
    schema_errors: SchemaErrorsItem,
    schema_storage_layout: SchemaStorageLayoutItem,
    factory: FactoryModuleItem
}

impl TryFrom<&ModuleStructIR> for ModuleStructItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            self_code: ModuleDefItem::try_from(ir)?,
            mod_item: ModuleModItem::try_from(ir)?,
            has_ident_item: HasIdentImplItem::try_from(ir)?,
            has_events_item: HasEventsImplItem::try_from(ir)?,
            schema_item: OdraModuleCfgItem::try_from(ir)?,
            schema_events_item: SchemaEventsItem::try_from(ir)?,
            schema_errors: SchemaErrorsItem::try_from(ir)?,
            schema_storage_layout: SchemaStorageLayoutItem::try_from(ir)?,
            factory: FactoryModuleItem::try_from(ir)?
        })
    }
}

impl ToTokens for ModuleStructItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            self_code,
            mod_item,
            has_ident_item,
            has_events_item,
            schema_item,
            schema_events_item,
            schema_errors,
            schema_storage_layout,
            factory
        } = self;
        tokens.extend(quote! {
            #self_code
            #mod_item
            #has_ident_item
            #has_events_item
            #schema_item
            #schema_events_item
            #schema_errors
            #schema_storage_layout
            #factory
        });
    }
}
