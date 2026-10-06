#![doc = "Binary for building wasm files from odra contracts."]
#![no_std]
#![cfg_attr(target_arch = "wasm32", no_main)]
#![allow(unused_imports, clippy::single_component_path_imports)]
// Contracts from a dependency crate (`fqn = "other_crate::module::Contract"` in Odra.toml) link their
// entry points only if this crate is referenced here: add `use other_crate;` next to the line below.
use flapper;

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
