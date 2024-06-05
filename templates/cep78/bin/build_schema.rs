#![doc = "Binary for building schema definitions from odra contracts."]
#[allow(unused_imports)]
use {{project-name}};

#[cfg(not(target_arch = "wasm32"))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
    fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    odra_build::schema(unsafe { crate::module_schema() }, unsafe {
        crate::casper_contract_schema()
    });
}
