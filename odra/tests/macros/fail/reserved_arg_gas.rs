use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u64>
}

#[odra::module]
impl Contract {
    pub fn set(&mut self, gas: u64) {
        self.value.set(gas);
    }
}

fn main() {}
