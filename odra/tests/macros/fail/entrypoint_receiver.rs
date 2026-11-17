//! An entry point is called on the contract: it takes `&self` or `&mut self`.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn consume(self) -> u32 {
        0
    }

    pub fn consume_mut(mut self) -> u32 {
        0
    }

    pub fn boxed(self: Box<Self>) -> u32 {
        0
    }

    pub fn associated() -> u32 {
        0
    }

    pub fn typed(self: &Self) -> u32 {
        0
    }
}

fn main() {}
