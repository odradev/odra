use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    #[odra(offchain)]
    pub fn bump(&mut self) {
        self.value.set(1);
    }
}

fn main() {}
