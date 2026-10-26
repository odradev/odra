use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    #[odra(offchain)]
    pub fn constant() -> u32 {
        1
    }
}

fn main() {}
