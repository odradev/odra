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

    #[odra(offchain)]
    pub fn bump_typed(self: &mut Self) {
        self.value.set(2);
    }

    #[odra(offchain)]
    #[allow(unused_mut)]
    pub fn read_mut_binding(mut self: &Self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
