//! `#[odra(..)]` is an attribute of module functions only.
use odra::prelude::*;

#[odra::module]
pub struct Contract {
    #[odra(using = "shared")]
    value: Var<u32>
}

#[odra::odra_type]
pub struct Type {
    #[odra(payable)]
    pub value: u32
}

fn main() {}
