#![doc = "Binary for building schema definitions from odra contracts."]
#![allow(unused_imports, redundant_imports)]
#![allow(clippy::single_component_path_imports)]
use {{project-name}};

// Links the dependency crates that Odra.toml lists contracts of
// (`fqn = "other_crate::module::Contract"`), so their entry points end up in the wasm.
odra::link_contract_crates!();

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
