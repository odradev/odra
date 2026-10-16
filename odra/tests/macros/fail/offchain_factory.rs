use odra::prelude::*;

#[odra::module(factory = on)]
pub struct Contract {
    value: Var<u32>
}

#[odra::module(factory = on)]
impl Contract {
    #[odra(offchain)]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
