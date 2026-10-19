//! Entry point names that clash with the functions of the generated refs and factories.
use odra::prelude::*;

#[odra::module]
pub struct Reserved0 {
    value: Var<u32>
}

#[odra::module]
impl Reserved0 {
    pub fn new(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Reserved1 {
    value: Var<u32>
}

#[odra::module]
impl Reserved1 {
    pub fn env(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Reserved2 {
    value: Var<u32>
}

#[odra::module]
impl Reserved2 {
    pub fn address(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Reserved3 {
    value: Var<u32>
}

#[odra::module]
impl Reserved3 {
    pub fn new_contract(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Reserved4 {
    value: Var<u32>
}

#[odra::module]
impl Reserved4 {
    pub fn upgrade_child_contract(&self) -> u32 {
        self.value.get_or_default()
    }
}

#[odra::module]
pub struct Reserved5 {
    value: Var<u32>
}

#[odra::module]
impl Reserved5 {
    pub fn batch_upgrade_child_contract(&self) -> u32 {
        self.value.get_or_default()
    }
}

fn main() {}
