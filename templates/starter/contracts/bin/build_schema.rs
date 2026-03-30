#![doc = "Binary for building schema definitions from odra contracts."]
#![allow(unused_imports, redundant_imports)]
#![allow(clippy::single_component_path_imports)]
use contracts;

#[cfg(all(not(odra_module = ""), not(target_arch = "wasm32")))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
    fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema;
}

#[cfg(all(not(odra_module = ""), not(target_arch = "wasm32")))]
fn main() {
    odra_build::schema(unsafe { crate::module_schema() }, unsafe {
        crate::casper_contract_schema()
    });
}

#[cfg(any(odra_module = "", target_arch = "wasm32"))]
fn main() {}
