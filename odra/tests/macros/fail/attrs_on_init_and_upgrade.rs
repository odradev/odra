//! `init` and `upgrade` get dedicated entry points that ignore `#[odra(..)]` flags.
use odra::prelude::*;

#[odra::module]
pub struct Contract0 {
    value: Var<u32>
}

#[odra::module]
impl Contract0 {
    #[odra(payable)]
    pub fn init(&mut self) {}
}

#[odra::module]
pub struct Contract1 {
    value: Var<u32>
}

#[odra::module]
impl Contract1 {
    #[odra(non_reentrant)]
    pub fn init(&mut self) {}
}

#[odra::module]
pub struct Contract2 {
    value: Var<u32>
}

#[odra::module]
impl Contract2 {
    #[odra(payable)]
    pub fn upgrade(&mut self) {}
}

#[odra::module]
pub struct Contract3 {
    value: Var<u32>
}

#[odra::module]
impl Contract3 {
    #[odra(non_reentrant)]
    pub fn upgrade(&mut self) {}
}

fn main() {}
