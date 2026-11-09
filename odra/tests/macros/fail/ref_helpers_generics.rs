use odra::prelude::*;

#[odra::module]
pub struct Token {
    value: Var<u32>
}

#[odra::module]
impl Token {
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::ref_helpers]
impl<T> Token {
    pub fn doubled(&self) -> u32 {
        self.value() * 2
    }
}

#[odra::ref_helpers]
impl Token<u32> {
    pub fn tripled(&self) -> u32 {
        self.value() * 3
    }
}

#[odra::ref_helpers]
impl (Token, Token) {
    pub fn quadrupled(&self) -> u32 {
        4
    }
}

#[odra::ref_helpers]
fn not_an_impl() {}

fn main() {}
