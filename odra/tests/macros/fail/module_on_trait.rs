//! `#[odra::module]` on a trait: the trait describes nothing the macro can generate for.
use odra::prelude::*;

#[odra::module]
pub trait Token {
    fn total_supply(&self) -> u32;
}

#[odra::factory]
pub trait TokenFactory {
    fn total_supply(&self) -> u32;
}

fn main() {}
