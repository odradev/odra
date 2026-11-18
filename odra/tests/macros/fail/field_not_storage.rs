//! A module field is a storage component; a plain value, and a non-module in a `SubModule`,
//! are reported in those terms.
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

fn main() {}
