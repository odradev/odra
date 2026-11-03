//! A module implementing a trait named by a path, not a bare ident. The trait is in scope, as
//! the generated code calls its methods on the module.
#![deny(warnings)]

use odra::prelude::*;
use traits::Counter;

pub mod traits {
    pub trait Counter {
        fn init(&mut self, value: u32);
        fn get(&self) -> u32;
    }
}

#[odra::module]
pub struct PathCounter {
    value: Var<u32>
}

#[odra::module]
impl traits::Counter for PathCounter {
    fn init(&mut self, value: u32) {
        self.value.set(value);
    }

    fn get(&self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
