//! An entry point argument is `name: Type`; a pattern has no name to put in the schema.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn set(&mut self, (a, b): (u32, u32)) {
        self.value.set(a + b);
    }
}

fn main() {}
