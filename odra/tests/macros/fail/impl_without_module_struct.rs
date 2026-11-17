//! `#[odra::module]` on the impl block of a struct that has no `#[odra::module]`: the first
//! error says the struct is not a module.
use odra::prelude::*;

pub struct NotAModule {
    value: Var<u32>
}

#[odra::module]
impl NotAModule {
    pub fn get(&self) -> u32 {
        0
    }
}

fn main() {}
