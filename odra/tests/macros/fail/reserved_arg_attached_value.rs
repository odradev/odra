use odra::casper_types::U512;
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<U512>
}

#[odra::module]
impl Contract {
    #[odra(payable)]
    pub fn deposit(&mut self, attached_value: U512) {
        self.value.set(attached_value);
    }
}

fn main() {}
