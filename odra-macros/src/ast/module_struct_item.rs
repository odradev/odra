use derive_try_from_ref::TryFromRef;
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
#[derive(TryFromRef)]
#[source(ModuleStructIR)]
#[err(syn::Error)]
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
