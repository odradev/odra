//! A type with no `CLType` and no serialization cannot be returned by an entry point; the errors
//! point at the type.
use odra::prelude::*;

pub struct NotRet;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn get(&self) -> NotRet {
        NotRet
    }
}

fn main() {}
