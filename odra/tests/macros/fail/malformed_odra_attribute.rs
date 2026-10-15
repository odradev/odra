//! `#[odra(..)]` takes a list of bare flags.
use odra::prelude::*;

#[odra::module]
pub struct WithValue {
    value: Var<u32>
}

#[odra::module]
impl WithValue {
    #[odra(payable = true)]
    pub fn deposit(&mut self) {
        self.value.set(1);
    }
}

#[odra::module]
pub struct WithList {
    value: Var<u32>
}

#[odra::module]
impl WithList {
    #[odra(offchain(host))]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct NoList {
    value: Var<u32>
}

#[odra::module]
impl NoList {
    #[odra]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct EmptyList {
    value: Var<u32>
}

#[odra::module]
impl EmptyList {
    #[odra()]
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct NameValue {
    value: Var<u32>
}

#[odra::module]
impl NameValue {
    #[odra = "payable"]
    pub fn deposit(&mut self) {
        self.value.set(1);
    }
}

#[odra::module]
pub struct NotAMeta {
    value: Var<u32>
}

#[odra::module]
impl NotAMeta {
    #[odra(payable;)]
    pub fn deposit(&mut self) {
        self.value.set(1);
    }
}

fn main() {}
