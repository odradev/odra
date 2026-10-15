//! An unknown `#[odra(..)]` argument is an error wherever the attribute is accepted.
use odra::prelude::*;

#[odra::module]
pub struct PubFn {
    value: Var<u32>
}

#[odra::module]
impl PubFn {
    #[odra(paybale)]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct PrivateFn {
    value: Var<u32>
}

#[odra::module]
impl PrivateFn {
    #[odra(view)]
    fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct FactoryFn {
    value: Var<u32>
}

#[odra::module(factory = on)]
impl FactoryFn {
    #[odra(payable, nonreentrant)]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

pub trait Value {
    fn value(&self) -> u32;
}

#[odra::module]
pub struct TraitImpl {
    value: Var<u32>
}

#[odra::module]
impl Value for TraitImpl {
    #[odra(off_chain)]
    fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::external_contract]
pub trait External {
    #[odra(payable, using = "x")]
    fn deposit(&mut self);
}

fn main() {}
