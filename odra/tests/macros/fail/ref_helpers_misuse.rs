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

#[odra::ref_helpers(extra)]
impl Token {
    pub fn doubled(&self) -> u32 {
        self.value() * 2
    }
}

pub trait Doubled {
    fn doubled(&self) -> u32;
}

#[odra::ref_helpers]
impl Doubled for Token {
    fn doubled(&self) -> u32 {
        self.value() * 2
    }
}

fn main() {}
