//! `#[odra::module(..)]` on a trait is `#[odra::module]` on a trait: rejected as such, whatever
//! the arguments.
use odra::prelude::*;

#[odra::module(events = [Transfer], factory = on)]
pub trait Token {
    fn total_supply(&self) -> u32;
}

fn main() {}
