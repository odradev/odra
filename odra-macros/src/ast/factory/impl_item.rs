use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    ast::{
        blueprint::BlueprintItem,
        contract_ref_item::RefItem,
        exec_parts::ModuleExecPartsItem,
        factory::parts::{FactoryContractItem, FactoryHasEntrypointsImplItem},
        parts_utils::OdraModuleCfgItem,
        schema::{FactorySchemaEntrypointsItem, SchemaCustomTypesItem},
        test_parts::{TestPartsItem, TestPartsReexportItem},
        wasm_parts::ModuleWasmPartsItem,
        Target
    },
    ModuleImplIR
};

/// Emits everything `#[odra::factory]` adds to a factory impl block, in this order.
pub struct FactoryModuleImplItem {
    self_code: TokenStream,
    has_entrypoints_item: FactoryHasEntrypointsImplItem,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: ModuleExecPartsItem,
    wasm_parts: OdraModuleCfgItem<ModuleWasmPartsItem>,
    contract_item: FactoryContractItem,
    blueprint: OdraModuleCfgItem<BlueprintItem>,
    schema_entrypoints: FactorySchemaEntrypointsItem,
    schema_custom_types: SchemaCustomTypesItem
}

impl TryFrom<&ModuleImplIR> for FactoryModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            self_code: ir.self_code(),
            has_entrypoints_item: FactoryHasEntrypointsImplItem::try_from(ir)?,
            ref_item: RefItem::try_new(ir, Target::Factory)?,
            test_parts: TestPartsItem::try_new(ir, Target::Factory)?,
            test_parts_reexport: TestPartsReexportItem::try_from(ir)?,
            exec_parts: ModuleExecPartsItem::try_new(ir, Target::Factory)?,
            wasm_parts: OdraModuleCfgItem(ModuleWasmPartsItem::try_new(ir, Target::Factory)?),
            contract_item: FactoryContractItem::try_from(ir)?,
            blueprint: OdraModuleCfgItem::try_from(ir)?,
            schema_entrypoints: FactorySchemaEntrypointsItem::try_from(ir)?,
            schema_custom_types: SchemaCustomTypesItem::try_from(ir)?
        })
    }
}

impl ToTokens for FactoryModuleImplItem {
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
            schema_custom_types
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
        });
    }
}

#[cfg(test)]
mod tests;
