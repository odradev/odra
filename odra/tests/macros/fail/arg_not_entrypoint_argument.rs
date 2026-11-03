//! A type with no `CLType` and no serialization cannot be an entry point argument;
//! every error points at the type, with the `EntrypointArgument` hint.
use odra::prelude::*;

pub struct NotAnArg;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn set(&mut self, value: NotAnArg) {
        let _ = value;
    }
}

fn main() {}
