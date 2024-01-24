use crate::ast::blueprint::BlueprintItem;
use crate::ast::entrypoints_item::HasEntrypointsImplItem;
use crate::ast::exec_parts::ExecPartsItem;
use crate::ast::ref_item::RefItem;
use crate::ast::test_parts::{TestPartsItem, TestPartsReexportItem};
use crate::ast::wasm_parts::WasmPartsModuleItem;
use crate::ir::ModuleImplIR;
use derive_try_from_ref::TryFromRef;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct ModuleImplItem {
    #[expr(input.self_code())]
    self_code: proc_macro2::TokenStream,
    has_entrypoints_item: HasEntrypointsImplItem,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: ExecPartsItem,
    wasm_parts: WasmPartsModuleItem,
    blueprint: BlueprintItem
}
