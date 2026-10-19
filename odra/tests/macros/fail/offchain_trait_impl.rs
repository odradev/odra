use odra::prelude::*;

pub trait Value {
    fn value(&self) -> u32;
}

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Value for Contract {
    #[odra(offchain)]
    fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
