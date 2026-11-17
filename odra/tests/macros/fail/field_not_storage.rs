//! A module field is a storage component; a plain value or a non-module in a `SubModule` is
//! reported in those terms. The second module's impl has no `#[odra::module]` struct.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    count: u32,
    name: String,
    inner: SubModule<u32>
}

#[odra::module]
impl Contract {
    pub fn get(&self) -> u32 {
        0
    }
}

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
