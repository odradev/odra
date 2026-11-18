//! A module, its impl block and an entry point are one concrete type each.
use odra::prelude::*;

#[odra::module]
pub struct Generic<T> {
    value: Var<T>
}

#[odra::module]
impl<T> Generic<T> {
    pub fn get(&self) -> u32 {
        0
    }
}

#[odra::module]
pub struct Bounded
where
    u32: Copy
{
    value: Var<u32>
}

#[odra::module]
impl Bounded
where
    u32: Copy
{
    pub fn get(&self) -> u32 {
        0
    }
}

#[odra::module]
pub struct Contract {
    value: Var<u32>
}

#[odra::module]
impl Contract {
    pub fn set<T: Into<u32>>(&mut self, value: T) {
        self.value.set(value.into());
    }

    pub fn set_impl(&mut self, value: impl Into<u32>) {
        self.value.set(value.into());
    }

    pub fn get<'a>(&'a self) -> u32 {
        0
    }

    pub fn get_where(&self) -> u32
    where
        u32: Copy
    {
        0
    }
}

fn main() {}
