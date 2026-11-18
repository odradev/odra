//! An argument is read from the call: `&mut T` has nothing to write to, and `&str` is a
//! `String` on the wire.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn set_name(&mut self, name: &str) {
        let _ = name;
    }

    pub fn set_value(&mut self, value: &mut u32) {
        let _ = value;
    }
}

fn main() {}
