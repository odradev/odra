use crate::ast::blueprint::BlueprintItem;
use crate::ast::contract_item::ContractItem;
use crate::ast::contract_ref_item::RefItem;
use crate::ast::entrypoints_item::HasEntrypointsImplItem;
use crate::ast::exec_parts::ModuleExecPartsItem;
use crate::ast::factory::FactoryModuleImplItem;
use crate::ast::parts_utils::OdraModuleCfgItem;
use crate::ast::schema::{SchemaCustomTypesItem, SchemaEntrypointsItem};
use crate::ast::test_parts::{TestPartsItem, TestPartsReexportItem};
use crate::ast::wasm_parts::ModuleWasmPartsItem;
use crate::ir::ModuleImplIR;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// Emits everything `#[odra::module]` adds to a module impl or trait block, in this order.
pub struct ModuleImplItem {
    self_code: TokenStream,
    has_entrypoints_item: HasEntrypointsImplItem,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: ModuleExecPartsItem,
    wasm_parts: OdraModuleCfgItem<ModuleWasmPartsItem>,
    contract_item: ContractItem,
    blueprint: OdraModuleCfgItem<BlueprintItem>,
    schema_entrypoints: SchemaEntrypointsItem,
    schema_custom_types: SchemaCustomTypesItem,
    factory: FactoryModuleImplItem
}

impl TryFrom<&ModuleImplIR> for ModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            self_code: ir.self_code(),
            has_entrypoints_item: HasEntrypointsImplItem::try_from(ir)?,
            ref_item: RefItem::try_from(ir)?,
            test_parts: TestPartsItem::try_from(ir)?,
            test_parts_reexport: TestPartsReexportItem::try_from(ir)?,
            exec_parts: ModuleExecPartsItem::try_from(ir)?,
            wasm_parts: OdraModuleCfgItem::try_from(ir)?,
            contract_item: ContractItem::try_from(ir)?,
            blueprint: OdraModuleCfgItem::try_from(ir)?,
            schema_entrypoints: SchemaEntrypointsItem::try_from(ir)?,
            schema_custom_types: SchemaCustomTypesItem::try_from(ir)?,
            factory: FactoryModuleImplItem::try_from(ir)?
        })
    }
}

impl ToTokens for ModuleImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            self_code,
            has_entrypoints_item,
            ref_item,
            test_parts,
            test_parts_reexport,
            exec_parts,
            wasm_parts,
            contract_item,
            blueprint,
            schema_entrypoints,
            schema_custom_types,
            factory
        } = self;
        tokens.extend(quote! {
            #self_code
            #has_entrypoints_item
            #ref_item
            #test_parts
            #test_parts_reexport
            #exec_parts
            #wasm_parts
            #contract_item
            #blueprint
            #schema_entrypoints
            #schema_custom_types
            #factory
        });
    }
}
