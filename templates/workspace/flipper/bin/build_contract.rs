#![doc = "Binary for building wasm files from odra contracts."]
#![no_std]
#![cfg_attr(target_arch = "wasm32", no_main)]
#![allow(unused_imports, clippy::single_component_path_imports)]
use flipper;

// Links the dependency crates that Odra.toml lists contracts of
// (`fqn = "other_crate::module::Contract"`), so their entry points end up in the wasm.
odra::link_contract_crates!();

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
