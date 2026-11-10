//! Every mistake of an impl block is reported in one build, not one per build.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn new(&self) -> u32 {
        self.value.get_or_default()
    }

    pub fn address(&self, gas: u64) -> u64 {
        gas
    }

    #[odra(offchain)]
    pub fn bump(&mut self) {
        self.value.set(1);
    }
}

fn main() {}
