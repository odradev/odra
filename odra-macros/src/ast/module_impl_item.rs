use crate::ast::blueprint::BlueprintItem;
use crate::ast::contract_item::ContractItem;
use crate::ast::contract_ref_item::RefItem;
use crate::ast::entrypoints_item::HasEntrypointsImplItem;
use crate::ast::exec_parts::ModuleExecPartsItem;
use crate::ast::factory::FactoryModuleImplItem;
use crate::ast::schema::{SchemaCustomTypesItem, SchemaEntrypointsItem};
use crate::ast::test_parts::{TestPartsItem, TestPartsReexportItem};
use crate::ast::wasm_parts::ModuleWasmPartsItem;
use crate::ir::ModuleImplIR;

#[derive(syn_derive::ToTokens)]
pub struct ModuleImplItem {
    self_code: proc_macro2::TokenStream,
    has_entrypoints_item: HasEntrypointsImplItem,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: ModuleExecPartsItem,
    wasm_parts: ModuleWasmPartsItem,
    contract_item: ContractItem,
    blueprint: BlueprintItem,
    schema_entrypoints: SchemaEntrypointsItem,
    schema_custom_types: SchemaCustomTypesItem,
    factory: FactoryModuleImplItem
}

impl TryFrom<&ModuleImplIR> for ModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            self_code: ir.self_code()?,
            has_entrypoints_item: HasEntrypointsImplItem::try_from(ir)?,
            ref_item: RefItem::try_from(ir)?,
            test_parts: TestPartsItem::try_from(ir)?,
            test_parts_reexport: TestPartsReexportItem::try_from(ir)?,
            exec_parts: ModuleExecPartsItem::try_from(ir)?,
            wasm_parts: ModuleWasmPartsItem::try_from(ir)?,
            contract_item: ContractItem::try_from(ir)?,
            blueprint: BlueprintItem::try_from(ir)?,
            schema_entrypoints: SchemaEntrypointsItem::try_from(ir)?,
            schema_custom_types: SchemaCustomTypesItem::try_from(ir)?,
            factory: FactoryModuleImplItem::try_from(ir)?
        })
    }
}
