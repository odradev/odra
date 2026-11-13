#![doc = "Binary for building wasm files from odra contracts."]
#![no_std]
#![cfg_attr(target_arch = "wasm32", no_main)]
#![allow(unused_imports, clippy::single_component_path_imports)]
use odra_examples;

// Links the dependency crates that Odra.toml lists contracts of (`odra_modules`).
odra::link_contract_crates!();

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
