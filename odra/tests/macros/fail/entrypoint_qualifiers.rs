//! An entry point is a plain `fn`.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub async fn asynchronous(&self) -> u32 {
        0
    }

    pub unsafe fn unchecked(&self) -> u32 {
        0
    }

    pub extern "C" fn external(&self) -> u32 {
        0
    }
}

fn main() {}
