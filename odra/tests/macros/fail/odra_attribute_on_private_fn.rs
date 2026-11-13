//! `#[odra(..)]` flags are applied by the generated entry point, so on a function that is not
//! one they would do nothing.
use odra::prelude::*;

#[odra::module]
pub struct Contract0 {
    value: Var<u32>
}

#[odra::module]
impl Contract0 {
    pub fn get(&self) -> u32 {
        self.helper()
    }

    #[odra(non_reentrant)]
    fn helper(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Contract1 {
    value: Var<u32>
}

#[odra::module]
impl Contract1 {
    #[odra(payable)]
    fn deposit(&mut self) {}

    #[odra(offchain)]
    pub(crate) fn total(&self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
