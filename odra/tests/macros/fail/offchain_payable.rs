use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    #[odra(offchain, payable)]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Guarded {
    value: Var<u32>
}

#[odra::module]
impl Guarded {
    #[odra(offchain, non_reentrant)]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
