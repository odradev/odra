//! A delegated function and a written one of the same name.
use odra::prelude::*;

#[odra::module]
pub struct Inner {
    value: Var<u32>
}

#[odra::module]
impl Inner {
    pub fn get(&self) -> u32 {
        0
    }
}

#[odra::module]
pub struct Contract {
    inner: SubModule<Inner>,
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn get(&self) -> u32 {
        0
    }

    delegate! {
        to self.inner {
            fn get(&self) -> u32;
        }
    }
}

fn main() {}
