//! `List` is a storage component with no serialization, so it cannot be a `Mapping` key.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    values: Mapping<List<u32>, u32>
}

#[odra::module]
impl Contract {
    pub fn get(&self) -> u32 {
        0
    }
}

fn main() {}
