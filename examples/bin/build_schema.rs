#![doc = "Binary for building schema definitions from odra contracts."]
#[allow(unused_imports, clippy::single_component_path_imports)]
use odra_examples;

#[cfg(all(not(odra_module = ""), not(target_arch = "wasm32")))]
extern "Rust" {
    fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema;
}

#[cfg(all(not(odra_module = ""), not(target_arch = "wasm32")))]
fn main() {
    odra_build::contract_schema(unsafe { crate::casper_contract_schema() });
}

#[cfg(any(odra_module = "", target_arch = "wasm32"))]
fn main() {}
