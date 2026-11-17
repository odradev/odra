//! `#[odra(..)]` is an attribute of module functions only.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    #[odra(using = "shared")]
    value: Var<u32>
}

#[odra::module]
#[odra(non_reentrant)]
impl Contract {
    pub fn get(&self) -> u32 {
        0
    }

    #[odra(payable)]
    const LIMIT: u32 = 1;
}

#[odra::odra_type]
pub struct Type {
    #[odra(payable)]
    pub value: u32
}

#[odra::odra_error]
pub enum Error {
    #[odra(payable)]
    Fail = 1
}

#[odra::event]
pub struct Transfer {
    #[odra(payable)]
    pub amount: u32
}

fn main() {}
