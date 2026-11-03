use derive_try_from_ref::TryFromRef;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    ast::{
        blueprint::BlueprintItem,
        parts_utils::OdraModuleCfgItem,
        factory::parts::{
            FactoryContractItem, FactoryExecPartsItem, FactoryHasEntrypointsImplItem,
            FactoryRefItem, FactoryTestPartsItem, FactoryWasmPartsItem
        },
        schema::{SchemaCustomTypesItem, FactorySchemaEntrypointsItem},
        test_parts::TestPartsReexportItem
    },
    ModuleImplIR
};

/// Emits everything `#[odra::factory]` adds to a factory impl block, in this order.
#[derive(TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct FactoryModuleImplItem {
    #[expr(input.self_code())]
    self_code: TokenStream,
    has_entrypoints_item: FactoryHasEntrypointsImplItem,
    ref_item: FactoryRefItem,
    test_parts: FactoryTestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: FactoryExecPartsItem,
    wasm_parts: OdraModuleCfgItem<FactoryWasmPartsItem>,
    contract_item: FactoryContractItem,
    blueprint: OdraModuleCfgItem<BlueprintItem>,
    schema_entrypoints: FactorySchemaEntrypointsItem,
    schema_custom_types: SchemaCustomTypesItem
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
