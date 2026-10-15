//! An `#[odra(..)]` argument may be given once per function, also across attributes.
use odra::prelude::*;

#[odra::module]
pub struct SameAttribute {
    value: Var<u32>
}

#[odra::module]
impl SameAttribute {
    #[odra(payable, payable)]
    pub fn deposit(&mut self) {
        self.value.set(1);
    }
}

#[odra::module]
pub struct TwoAttributes {
    value: Var<u32>
}

#[odra::module]
impl TwoAttributes {
    #[odra(payable)]
    #[odra(non_reentrant, payable)]
    pub fn deposit(&mut self) {
        self.value.set(1);
    }
}

#[odra::module]
pub struct Offchain {
    value: Var<u32>
}

#[odra::module]
impl Offchain {
    #[odra(offchain)]
    #[odra(offchain)]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct FactoryFn {
    value: Var<u32>
}

#[odra::module(factory = on)]
impl FactoryFn {
    #[odra(non_reentrant, non_reentrant)]
    pub fn set(&mut self) {
        self.value.set(1);
    }
}

fn main() {}
